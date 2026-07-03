$ErrorActionPreference = 'Stop'

$root   = 'C:\Projects\Segovia'
$py     = Join-Path $root '.venv-si\Scripts\python.exe'
$script = Join-Path $root 'bench\replay_latency_si.py'
$cbin   = Join-Path $root 'tests\data\_spikeglx_ephysData_g0_t0.imec0.ap.cbin'
$outDir = Join-Path $root 'bench\_tmp'
if (-not (Test-Path $outDir)) { New-Item -ItemType Directory -Force -Path $outDir | Out-Null }

$stamp  = Get-Date -Format 'yyyyMMdd-HHmmss'
$base   = Join-Path $outDir "si_300ms_full_$stamp"
$out    = "$base.out"
$errlog = "$base.err"
$hb     = "$base.heartbeat"
$status = "$base.status"
$lock   = Join-Path $outDir 'si_300ms_full.lock'
$result = Join-Path $outDir 'si_headline_full_result.jsonl'

function Write-HB($m) {
    Add-Content -Path $hb -Value ("{0}  {1}" -f (Get-Date -Format 'HH:mm:ss'), $m)
}

foreach ($p in @($py, $script, $cbin)) {
    if (-not (Test-Path $p)) {
        Set-Content $status "FAILED prerequisite-missing $p"
        return
    }
}

if (Test-Path $lock) {
    $oldpid = Get-Content $lock -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($oldpid -and (Get-Process -Id ([int]$oldpid) -ErrorAction SilentlyContinue)) {
        Set-Content $status "FAILED already-running pid=$oldpid"
        return
    }
}

Write-HB 'preflight ok'
$cpu = (Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
Write-HB ("cpu-load-at-start {0:N1}%" -f $cpu)
if ($cpu -gt 50) {
    Set-Content $status ("ABORTED cpu-busy {0:N1}%" -f $cpu)
    Write-HB ("aborted: cpu {0:N1}% over 50% would taint latencies" -f $cpu)
    return
}

$startTime = Get-Date
Set-Content $status ("RUNNING started={0}" -f $startTime.ToString('HH:mm:ss'))
Write-HB 'launching SI 300ms full-length leg'

$argList = @($script, '--kind', 'cbin', '--cbin', $cbin, '--load-sync', '--chunk-samples', '9000')
$proc = Start-Process -FilePath $py -ArgumentList $argList `
    -RedirectStandardOutput $out -RedirectStandardError $errlog -PassThru -WindowStyle Hidden
Set-Content $lock $proc.Id
Write-HB ("pid={0}" -f $proc.Id)

$ErrorActionPreference = 'Continue'
while (-not $proc.HasExited) {
    Start-Sleep -Seconds 30
    $rss = 0
    try { $rss = (Get-Process -Id $proc.Id -ErrorAction Stop).WorkingSet64 / 1GB } catch {}
    $elapsed = ((Get-Date) - $startTime).TotalMinutes
    Write-HB ("alive {0:N1} min  rss={1:N2} GB" -f $elapsed, $rss)
}

$proc.WaitForExit()
$code = $proc.ExitCode
$endTime = Get-Date
Remove-Item $lock -ErrorAction SilentlyContinue

if ($code -eq 0) {
    $json = Get-Content $out -ErrorAction SilentlyContinue | Where-Object { $_.Trim().StartsWith('{') } | Select-Object -First 1
    if ($json) { Set-Content $result $json }
    Set-Content $status ("DONE exit=0 elapsed={0:N1}min end={1}" -f (($endTime - $startTime).TotalMinutes), $endTime.ToString('HH:mm:ss'))
    Write-HB 'DONE clean exit 0'
} else {
    Set-Content $status ("FAILED exit={0} end={1}" -f $code, $endTime.ToString('HH:mm:ss'))
    Write-HB ("FAILED exit={0}" -f $code)
}
