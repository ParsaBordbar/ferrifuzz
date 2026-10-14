import pytest

import ferrifuzz as ff


@pytest.fixture
def temp():
    v = ff.Variable("Temperature", min_value=0.0, max_value=40.0)
    v["Cold"] = ff.mf.triangle(0.0, 0.0, 20.0)
    v["Warm"] = ff.mf.triangle(10.0, 20.0, 30.0)
    v["Hot"] = ff.mf.triangle(20.0, 40.0, 40.0)
    return v


def test_variable(temp):
    assert temp.fuzzify(25.0) == {"Cold": 0.0, "Warm": 0.5, "Hot": 0.25}
    assert temp(25.0) == temp.fuzzify(25.0)
    assert temp.get_terms() == ["Cold", "Warm", "Hot"]
    assert repr(temp) == "Variable('Temperature', [0, 40], 3 terms)"
    assert temp["Hot"].membership(40.0) == 1.0
    assert "Hot" in temp and len(temp) == 3
    with pytest.raises(KeyError):
        temp["Missing"]
    with pytest.raises(TypeError):
        temp["Bad"] = 3.0
    with pytest.raises(ValueError):
        ff.Variable("x", 1.0, 1.0)


def test_variable_accepts_raw_shapes_and_replaces_terms(temp):
    temp["Hot"] = ff.TriangularMF(20.0, 30.0, 40.0)
    assert temp["Hot"].membership(30.0) == 1.0
    temp["Hot"] = ff.mf.it2(ff.mf.triangle(20, 30, 40), ff.mf.triangle(25, 30, 35))
    assert isinstance(temp["Hot"], ff.IT2MF)
    assert temp.get_terms() == ["Cold", "Warm", "Hot"]


def test_rule_parsing_round_trip():
    s = "IF t IS NOT very hot OR h IS low AND t IS cold THEN f IS slow AND a IS on"
    r = ff.Rule.from_string(s)
    assert str(r) == s
    assert r.group_count() == 2
    assert r.consequent_count() == 2
    assert r.conclusions == [("f", "slow"), ("a", "on")]
    c = r.clauses[0][0]
    assert (c.var, c.term, c.negated, c.hedge, c.hedge_power) == ("t", "hot", True, "very", 2.0)
    assert ff.Rule.from_string(s) == r


def test_rule_constructor():
    r = ff.Rule(conditions=[("t", "hot"), ("h", "high")], conclusion=("f", "fast"), weight=0.5)
    assert r.conditions == [("t", "hot"), ("h", "high")]
    assert r.conclusion == ("f", "fast")
    assert str(r) == "IF t IS hot AND h IS high THEN f IS fast WITH 0.5"
    assert ff.Rule.from_string(str(r)) == r
    r.weight = 1.0
    assert str(r) == "IF t IS hot AND h IS high THEN f IS fast"
    r2 = ff.Rule(antecedent=[[("a", "x")], [("b", "y")]], conclusion=[ff.Clause("o", "p", hedge="very")])
    assert str(r2) == "IF a IS x OR b IS y THEN o IS very p"
    with pytest.raises(ValueError):
        r2.conditions
    with pytest.raises(ValueError):
        ff.Rule(conclusion=("f", "fast"))
    with pytest.raises(ValueError):
        ff.Clause("t", "hot", hedge="nope")


@pytest.mark.parametrize(
    "bad",
    [
        "t IS hot THEN f IS fast",
        "IF t IS hot",
        "IF t IS hot THEN f IS a OR g IS b",
        "IF t IS really hot THEN f IS fast",
        "IF t IS very NOT hot THEN f IS fast",
        "IF (t IS hot THEN f IS fast",
        "IF t IS hot THEN f IS fast WITH 1.5",
    ],
)
def test_malformed_rules(bad):
    with pytest.raises(ff.RuleSyntaxError):
        ff.Rule.from_string(bad)


def test_rule_syntax_extras():
    r = ff.Rule.from_string("if (a is x or b is y) and c is NOT z then o is p with 0.5")
    assert r.group_count() == 2
    assert r.weight == 0.5
    assert str(r) == "IF a IS x AND c IS NOT z OR b IS y AND c IS NOT z THEN o IS p WITH 0.5"
    with pytest.raises(ValueError, match="really_hot"):
        ff.Rule.from_string("IF t IS really hot THEN f IS fast")


def test_operators():
    assert ff.Operator.t_norm(0.5, 0.4) == 0.4
    assert ff.Operator.t_norm(0.5, 0.4, kind="product") == pytest.approx(0.2)
    assert ff.Operator.t_conorm(0.5, 0.4, kind="probor") == pytest.approx(0.7)
    assert ff.Operator.t_norm((0.2, 0.6), (0.4, 0.5)) == (0.2, 0.5)
    assert ff.Operator.complement(0.3) == pytest.approx(0.7)
    assert ff.Operator.complement((0.2, 0.6)) == pytest.approx((0.4, 0.8))
    with pytest.raises(TypeError):
        ff.Operator.t_norm(0.5, (0.1, 0.2))
    with pytest.raises(ValueError, match="did you mean 'product'"):
        ff.Operator.t_norm(0.5, 0.4, kind="prodcut")
    with pytest.raises(ValueError):
        ff.Operator.complement(0.5, kind="yager", param=0)


def test_hedges():
    assert ff.Hedge.very()(0.8) == pytest.approx(0.64)
    assert ff.Hedge.somewhat()((0.36, 0.81)) == pytest.approx((0.6, 0.9))
    assert ff.Hedge(3.0)(0.5) == 0.125
    assert ff.Hedge.plus().power == 1.25
    assert ff.Hedge.extremely()(0.5) == 0.125


def test_automf_and_str():
    v = ff.Variable("e", -1, 1).automf(5)
    assert v.get_terms() == ["very_low", "low", "medium", "high", "very_high"]
    assert v.fuzzify(-1)["very_low"] == 1.0
    assert str(ff.Variable("x", 0, 2).automf(["a", "b"])) == "x [0, 2]: a = triangle(-2, 0, 2), b = triangle(0, 2, 4)"
    g = ff.Variable("g", 0, 1).automf(3, shape="gaussian")
    assert g["medium"].shape() == "Gaussian"
    with pytest.raises(ValueError):
        ff.Variable("g", 0, 1).automf(3, shape="square")
    assert list(v) == v.get_terms()


def test_fuzzify_includes_it2_terms():
    v = ff.Variable("t", 0, 10)
    v["a"] = ff.mf.triangle(0, 0, 10)
    v["b"] = ff.mf.it2(ff.mf.triangle(0, 5, 10), ff.mf.triangle(2, 5, 8))
    out = v.fuzzify(5)
    assert out["a"] == 0.5 and out["b"] == (1.0, 1.0)
