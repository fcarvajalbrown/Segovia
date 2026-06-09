import numpy as np
import segovia


def test_version_is_present():
    assert isinstance(segovia.__version__, str)
    assert segovia.__version__


def test_zeros_shape_dtype_values():
    a = segovia.zeros(4, 10)
    assert a.shape == (4, 10)
    assert a.dtype == np.int16
    assert not a.flags["OWNDATA"]
    assert a.base is not None
    assert not a.any()
