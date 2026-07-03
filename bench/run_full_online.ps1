$ErrorActionPreference = 'Stop'

$root   = 'C:\Projects\Segovia'
$seg    = Join-Path $root '.venv\Scripts\python.exe'
$si     = Join-Path $root '.venv-si\Scripts\python.exe'
$segScript = Join-Path $root 'bench\replay_latency.py'
$siScript  = Join-Path $root 'bench\replay_latency_si.py'
$cbin   = Join-Path $root 'tests\data\_spikeglx_ephysData_g0_t0.imec0.ap.cbin'
$ch     = Join-Path $root 'tests\data\_spikeglx_ephysData_g0_t0.imec0.ap.ch'
$outDir = Join-Path $root 'bench\_tmp'
if (-not (Test-Path $outDir)) { New-Item -ItemType Directory -Force -Path $outDir | Out-Null }

$stamp   = Get-Date -Format 'yyyyMMdd-HHmmss'
$base    = Join-Path $outDir "full_online_$stamp"
$hb      = "$base.heartbeat"
$status  = "$base.status"
$results = "$base.results.jsonl"
$lock    = Join-Path $outDir 'full_online.lock'

function Write-HB($m) {
    Add-Content -Path $hb -Value ("{0}  {1}" -f (Get-Date -Format 'HH:mm:ss'), $m)
}

foreach ($p in @($seg, $si, $segScript, $siScript, $cbin, $ch)) {
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
Set-Content $lock $PID

$cpu = (Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
Write-HB ("cpu-load-at-start {0:N1}%" -f $cpu)
if ($cpu -gt 50) {
    Set-Content $status ("ABORTED cpu-busy {0:N1}%" -f $cpu)
    Remove-Item $lock -ErrorAction SilentlyContinue
    return
}

$legs = @(
    @{ tag = 'segovia_cs30000';  exe = $seg; argList = @($segScript, '--kind', 'cbin', '--cbin', $cbin, '--ch', $ch, '--chunk-samples', '30000', '--json-only') },
    @{ tag = 'si_cs30000';       exe = $si;  argList = @($siScript,  '--kind', 'cbin', '--cbin', $cbin, '--load-sync', '--chunk-samples', '30000', '--json-only') },
    @{ tag = 'segovia_cs3000';   exe = $seg; argList = @($segScript, '--kind', 'cbin', '--cbin', $cbin, '--ch', $ch, '--chunk-samples', '3000', '--json-only') },
    @{ tag = 'si_cs3000';        exe = $si;  argList = @($siScript,  '--kind', 'cbin', '--cbin', $cbin, '--load-sync', '--chunk-samples', '3000', '--json-only') }
)

$ErrorActionPreference = 'Continue'
$startAll = Get-Date
Set-Content $status ("RUNNING started={0} legs={1}" -f $startAll.ToString('HH:mm:ss'), $legs.Count)

$done = 0
foreach ($leg in $legs) {
    $tag = $leg.tag
    $out = "$base.$tag.out"
    $err = "$base.$tag.err"
    Write-HB ("leg-start {0}" -f $tag)
    $legStart = Get-Date
    $proc = Start-Process -FilePath $leg.exe -ArgumentList $leg.argList `
        -RedirectStandardOutput $out -RedirectStandardError $err -PassThru -WindowStyle Hidden
    while (-not $proc.HasExited) {
        Start-Sleep -Seconds 30
        $rss = 0
        try { $rss = (Get-Process -Id $proc.Id -ErrorAction Stop).WorkingSet64 / 1GB } catch {}
        $elapsed = ((Get-Date) - $legStart).TotalMinutes
        Write-HB ("{0} alive {1:N1} min  rss={2:N2} GB" -f $tag, $elapsed, $rss)
    }
    $proc.WaitForExit()
    $code = $proc.ExitCode
    $json = Get-Content $out -ErrorAction SilentlyContinue | Where-Object { $_.Trim().StartsWith('{') } | Select-Object -First 1
    if ($json) {
        Add-Content $results ("{{`"tag`":`"{0}`",`"payload`":{1}}}" -f $tag, $json)
        $done += 1
        Write-HB ("leg-done {0} exit={1}" -f $tag, $code)
    } else {
        Write-HB ("leg-noresult {0} exit={1} (see {2})" -f $tag, $code, $err)
    }
}

$endAll = Get-Date
Remove-Item $lock -ErrorAction SilentlyContinue
Set-Content $status ("DONE legs-with-result={0}/{1} elapsed={2:N1}min end={3}" -f $done, $legs.Count, (($endAll - $startAll).TotalMinutes), $endAll.ToString('HH:mm:ss'))
Write-HB ("ALL DONE {0}/{1} legs" -f $done, $legs.Count)
