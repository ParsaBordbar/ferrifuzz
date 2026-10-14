import pytest

import ferrifuzz as ff


def test_triangle_shape():
    t = ff.mf.triangle(0.0, 5.0, 10.0)
    assert [t.membership(x) for x in (0, 2.5, 5, 7.5, 10)] == [0.0, 0.5, 1.0, 0.5, 0.0]
    assert t.center() == 5.0
    assert t.shape() == "Triangular"
    assert repr(t) == "triangle(0, 5, 10)"


@pytest.mark.parametrize(
    "mf, x",
    [
        (ff.mf.triangle(0.0, 0.0, 20.0), 0.0),
        (ff.mf.triangle(20.0, 40.0, 40.0), 40.0),
        (ff.mf.trapezoid(0.0, 0.0, 10.0, 20.0), 0.0),
        (ff.mf.trapezoid(0.0, 10.0, 20.0, 20.0), 20.0),
    ],
)
def test_shoulders_peak_at_their_edge(mf, x):
    assert mf.membership(x) == 1.0


def test_smooth_shapes():
    assert ff.mf.gaussian(5.0, 2.0).membership(5.0) == 1.0
    assert ff.mf.bell(0.0, 2.0, 3.0).membership(2.0) == pytest.approx(0.5)
    assert ff.mf.sigmoid(0.0, 1.0).membership(0.0) == pytest.approx(0.5)
    assert ff.mf.singleton(3.0).membership(3.0) == 1.0


@pytest.mark.parametrize(
    "build",
    [
        lambda: ff.mf.triangle(5, 0, 10),
        lambda: ff.mf.trapezoid(0, 3, 2, 4),
        lambda: ff.mf.gaussian(0, 0),
        lambda: ff.mf.bell(0, -1, 1),
        lambda: ff.TriangularMF(3, 2, 1),
    ],
)
def test_invalid_parameters_raise(build):
    with pytest.raises(ValueError):
        build()


def test_raw_shape_classes():
    raw = ff.TriangularMF(0.0, 5.0, 10.0)
    assert isinstance(raw, ff.TriangularMF)
    assert raw.membership(2.5) == 0.5
    assert ff.SingletonMF(3.0).membership(3.0) == 1.0
    assert isinstance(ff.mf.triangle(0, 1, 2), ff.Type1MF)


def test_it2():
    m = ff.mf.it2(ff.mf.triangle(0, 5, 10), ff.mf.triangle(2, 5, 8))
    lo, up = m.membership(3.0)
    assert lo == pytest.approx(1 / 3)
    assert up == pytest.approx(0.6)
    assert m.shapes() == ("Triangular", "Triangular")
    assert m.upper.membership(5.0) == 1.0
    assert repr(m) == "it2(triangle(0, 5, 10), triangle(2, 5, 8))"


def test_it2_checks_order_and_lower_height():
    wide, narrow = ff.mf.gaussian(5, 3), ff.mf.gaussian(5, 1)
    with pytest.raises(ValueError, match="swapped"):
        ff.mf.it2(narrow, wide)
    m = ff.mf.it2(wide, narrow, lower_height=0.5)
    assert m.membership(5.0) == (0.5, 1.0)
    assert m.lower_height == 0.5
    with pytest.raises(ValueError):
        ff.mf.it2(wide, narrow, lower_height=0)


def test_vectorised_membership():
    np = pytest.importorskip("numpy")
    t = ff.mf.triangle(0, 5, 10)
    out = t.membership(np.array([0.0, 2.5, 5.0]))
    assert isinstance(out, np.ndarray)
    assert out.tolist() == [0.0, 0.5, 1.0]
    assert t([2.5])[0] == 0.5
    lo, up = ff.mf.it2(ff.mf.triangle(0, 5, 10), ff.mf.triangle(2, 5, 8)).membership([5.0, 3.0])
    assert lo.tolist()[0] == 1.0 and up.tolist()[1] == pytest.approx(0.6)
    with pytest.raises(TypeError):
        t.membership("x")
    assert isinstance(ff.IT2MF(ff.mf.triangle(0, 5, 10), ff.mf.triangle(2, 5, 8)), ff.IT2MF)
