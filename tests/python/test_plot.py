import pytest

import ferrifuzz as ff

plt = pytest.importorskip("matplotlib.pyplot")
pytest.importorskip("numpy")


@pytest.fixture
def system():
    x = ff.Variable("x", 0, 10).automf(3)
    y = ff.Variable("y", 0, 10).automf(3)
    out = ff.Variable("out", 0, 1).automf(["lo", "hi"])
    s = ff.FuzzySystem()
    s.add_input(x)
    s.add_input(y)
    s.add_output(out)
    s.add_rules(["IF x IS low AND y IS low THEN out IS lo", "IF x IS high OR y IS high THEN out IS hi"])
    return s


def test_plots_draw(system):
    import ferrifuzz.plot as fp

    assert fp.plot_variable(system.input("x")).get_lines()
    assert fp.plot_output(system, {"x": 2, "y": 3}, "out").get_legend()
    assert fp.plot_response(system, "x", "out", fixed={"y": 1}).get_lines()
    assert fp.plot_surface(system, "x", "y", "out", n=8) is not None
    with pytest.raises(ff.InputError):
        fp.plot_response(system, "x", "out", fixed={"z": 1})
    it2 = ff.Variable("t", 0, 10)
    it2["a"] = ff.mf.it2(ff.mf.triangle(0, 5, 10), ff.mf.triangle(2, 5, 8), lower_height=0.6)
    assert fp.plot_variable(it2).collections
    plt.close("all")


def test_lazy_attribute():
    assert callable(ff.plot.plot_variable)
