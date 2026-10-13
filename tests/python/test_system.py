import math

import pytest

import ferrifuzz as ff


def tipper(**kwargs):
    service = ff.Variable("service", 0.0, 10.0)
    service["poor"] = ff.mf.trapezoid(0, 0, 2, 5)
    service["good"] = ff.mf.triangle(2, 5, 8)
    service["excellent"] = ff.mf.trapezoid(5, 8, 10, 10)
    tip = ff.Variable("tip", 0.0, 30.0)
    tip["low"] = ff.mf.triangle(0, 5, 10)
    tip["medium"] = ff.mf.triangle(10, 15, 20)
    tip["high"] = ff.mf.triangle(20, 25, 30)
    sys = ff.FuzzySystem(**kwargs)
    sys.add_input(service)
    sys.add_output(tip)
    for r in (
        "IF service IS poor THEN tip IS low",
        "IF service IS good THEN tip IS medium",
        "IF service IS excellent THEN tip IS high",
    ):
        sys += ff.Rule.from_string(r)
    return sys


def test_mamdani_basics():
    sys = tipper()
    assert sys.compute({"service": 5.0})["tip"] == pytest.approx(15.0, abs=0.01)
    assert sys({"service": 0.0})["tip"] == pytest.approx(5.0, abs=0.01)
    assert sys({"service": 10.0})["tip"] == pytest.approx(25.0, abs=0.01)
    assert sys.input_names() == ["service"] and sys.output_names() == ["tip"]
    assert sys.rule_count() == 3
    assert sys.validate() == []
    assert repr(sys) == (
        "FuzzySystem: Mamdani (defuzz=centroid, implication=mamdani); and=min, or=max; "
        "inputs [service] → outputs [tip]; 3 rules"
    )
    assert (sys.and_method, sys.defuzz_method, sys.type, sys.is_it2) == ("min", "centroid", "auto", False)


@pytest.mark.parametrize("method", ["centroid", "bisector", "mom", "som", "lom"])
def test_defuzzifiers_stay_in_universe(method):
    sys = tipper(defuzz_method=method)
    for s in range(11):
        assert 0.0 <= sys({"service": float(s)})["tip"] <= 30.0


def test_sugeno_weighted_average_and_linear_consequent():
    sys = tipper(defuzz_method="weighted_average")
    # service=3.5 → poor 0.5, good 0.5 → (5 + 15) / 2
    assert sys({"service": 3.5})["tip"] == pytest.approx(10.0)
    sys.add_consequent("tip", "high", {"service": 2.0}, bias=1.0)
    assert sys({"service": 10.0})["tip"] == pytest.approx(21.0)


def test_input_errors():
    sys = tipper()
    with pytest.raises(ff.InputError, match="missing input 'service'"):
        sys({})
    with pytest.raises(ff.InputError, match="did you mean 'service'"):
        sys({"servce": 1.0})
    with pytest.raises(ff.InputError, match="NaN"):
        sys({"service": float("nan")})
    with pytest.raises(ff.InputError):
        sys({"service": "high"})
    with pytest.raises(ff.InputError):
        sys.compute_batch({"service": [1.0], "other": [2.0]})


def test_out_of_range_inputs_are_clamped():
    sys = tipper()
    assert sys({"service": -5})["tip"] == sys({"service": 0})["tip"]
    raw = tipper(clamp_inputs=False)
    assert math.isnan(raw({"service": 50})["tip"])


def test_unknown_option_names_raise():
    with pytest.raises(ValueError, match="did you mean 'centroid'"):
        ff.FuzzySystem(defuzz_method="centriod")
    with pytest.raises(ValueError):
        ff.FuzzySystem(type_reduction="bogus")


def test_model_errors():
    sys = tipper()
    sys += "IF service IS gud THEN tip IS hihg"
    with pytest.raises(ff.ModelError) as e:
        sys({"service": 5})
    assert "did you mean 'good'" in str(e.value) and "did you mean 'high'" in str(e.value)
    sys = tipper()
    sys.add_consequent("tip", "high", {"service": 1.0})
    with pytest.raises(ff.ModelError, match="Sugeno"):
        sys.check()
    sys = tipper(defuzz_method="weighted_average")
    sys.add_consequent("tip", "high", {"servise": 1.0})
    with pytest.raises(ff.ModelError, match="did you mean 'service'"):
        sys.check()


def test_strings_rules_and_add_rules():
    sys = tipper()
    sys.add_rules(["IF service IS poor THEN tip IS medium WITH 0.2", ff.Rule.from_string("IF service IS good THEN tip IS high WITH 0.1")])
    assert sys.rule_count() == 5
    with pytest.raises(ff.RuleSyntaxError):
        sys.add_rules(["IF bad", "IF service IS poor THEN tip IS low"])
    assert sys.rule_count() == 5


def test_introspection():
    np = pytest.importorskip("numpy")
    sys = tipper()
    assert sys.rule_strengths({"service": 3.5}) == pytest.approx([0.5, 0.5, 0.0])
    xs, mus = sys.output_set({"service": 3.5}, "tip")
    assert len(xs) == len(mus) == 201 and float(np.max(mus)) == pytest.approx(0.5)
    assert sys.evaluate([5.0]) == pytest.approx([sys({"service": 5.0})["tip"]])
    assert sys.compute_interval({"service": 5.0})["tip"][0] == sys({"service": 5.0})["tip"]
    assert sys.input("service").name == "service"


def test_batch_matches_single_rows():
    sys = tipper()
    xs = [i / 2 for i in range(21)]
    out = sys.compute_batch({"service": xs})
    assert [float(v) for v in out["tip"]] == pytest.approx([sys({"service": x})["tip"] for x in xs])
    with pytest.raises(ValueError):
        sys.compute_batch({"service": [1.0, 2.0], "other": [1.0]})


def test_it2_auto_selected_and_degenerate_fou_matches_type1():
    t1 = tipper()
    it2 = tipper(type="it2")
    for s in (1.0, 4.0, 6.5, 9.0):
        assert it2({"service": s})["tip"] == pytest.approx(t1({"service": s})["tip"], abs=1e-6)

    temp = ff.Variable("temp", 0.0, 100.0)
    temp["hot"] = ff.mf.it2(ff.mf.triangle(30, 70, 100), ff.mf.triangle(50, 70, 90))
    fan = ff.Variable("fan", 0.0, 100.0)
    fan["fast"] = ff.mf.it2(ff.mf.triangle(40, 80, 100), ff.mf.triangle(60, 80, 100))
    sys = ff.FuzzySystem()
    sys.add_input(temp)
    sys.add_output(fan)
    sys += ff.Rule.from_string("IF temp IS hot THEN fan IS fast")
    lo, hi = sys.compute_interval({"temp": 70.0})["fan"]
    assert lo < hi
    xs, lower, upper = sys.output_fou({"temp": 70.0}, "fan")
    assert all(a <= b + 1e-12 for a, b in zip(lower, upper))
    with pytest.raises(ff.ModelError, match="type_reduction"):
        bad = ff.FuzzySystem(defuzz_method="bisector")
        bad.add_input(temp)
        bad.add_output(fan)
        bad += "IF temp IS hot THEN fan IS fast"
        bad.check()
    for tr in ("km", "cos", "height"):
        sys2 = ff.FuzzySystem(type_reduction=tr)
        sys2.add_input(temp)
        sys2.add_output(fan)
        sys2 += ff.Rule.from_string("IF temp IS hot THEN fan IS fast")
        assert 40.0 < sys2({"temp": 70.0})["fan"] < 100.0
    assert not math.isnan(sys({"temp": 70.0})["fan"])


def test_validate_reports_problems():
    sys = tipper()
    sys += ff.Rule.from_string("IF service IS amazing THEN bonus IS big")
    sys += ff.Rule.from_string("IF service IS poor THEN tip IS low")
    issues = "\n".join(sys.validate())
    assert "error: rule #4" in issues and "no term 'amazing'" in issues
    assert "unknown output variable 'bonus'" in issues
    assert "warning: rules #1 and #5 are duplicates" in issues
