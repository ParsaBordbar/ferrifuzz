//! The classic tipping problem: service and food quality → tip percentage.

use ferrifuzz::prelude::*;

fn main() -> Result<(), Error> {
    let service = Variable::new("service", 0.0, 10.0)?
        .with_term("poor", mf::gaussian(0.0, 1.5)?)
        .with_term("good", mf::gaussian(5.0, 1.5)?)
        .with_term("excellent", mf::gaussian(10.0, 1.5)?);
    let food = Variable::new("food", 0.0, 10.0)?
        .with_term("rancid", mf::trapezoid(0.0, 0.0, 1.0, 3.0)?)
        .with_term("delicious", mf::trapezoid(7.0, 9.0, 10.0, 10.0)?);
    let tip = Variable::new("tip", 0.0, 30.0)?
        .with_term("cheap", mf::triangle(0.0, 5.0, 10.0)?)
        .with_term("average", mf::triangle(10.0, 15.0, 20.0)?)
        .with_term("generous", mf::triangle(20.0, 25.0, 30.0)?);

    let mut system = FuzzySystem::new();
    system.add_input(service).add_input(food).add_output(tip);
    system.add_rules_str([
        "IF service IS poor OR food IS rancid THEN tip IS cheap",
        "IF service IS good THEN tip IS average",
        "IF service IS excellent OR food IS delicious THEN tip IS generous",
    ])?;
    assert!(system.validate().is_empty());

    println!("service  food    tip");
    for (s, f) in [(1.0, 2.0), (5.0, 5.0), (8.0, 6.0), (10.0, 10.0)] {
        let out = system.compute([("service", s), ("food", f)])?;
        println!("{:>7} {:>5} {:>6.2}%", s, f, out["tip"]);
    }
    Ok(())
}
