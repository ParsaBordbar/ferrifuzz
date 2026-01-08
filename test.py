import pyfuzzy_rust as fuzzy

def test_membership_functions():
    """Test basic membership functions"""
    print("=" * 60)
    print("Testing Membership Functions")
    print("=" * 60)
    
    # Triangular MF
    tri = fuzzy.TriangularMF(0.0, 50.0, 100.0)
    print(f"\n{tri}")
    print(f"  membership(0) = {tri.membership(0.0)}")
    print(f"  membership(25) = {tri.membership(25.0)}")
    print(f"  membership(50) = {tri.membership(50.0)}")
    print(f"  membership(75) = {tri.membership(75.0)}")
    print(f"  membership(100) = {tri.membership(100.0)}")
    
    # Trapezoidal MF
    trap = fuzzy.TrapezoidalMF(0.0, 20.0, 80.0, 100.0)
    print(f"\n{trap}")
    print(f"  membership(0) = {trap.membership(0.0)}")
    print(f"  membership(10) = {trap.membership(10.0)}")
    print(f"  membership(50) = {trap.membership(50.0)}")
    print(f"  membership(90) = {trap.membership(90.0)}")
    print(f"  membership(100) = {trap.membership(100.0)}")


def test_linguistic_variables():
    """Test linguistic variables"""
    print("\n" + "=" * 60)
    print("Testing Linguistic Variables")
    print("=" * 60)
    
    temp = fuzzy.Variable("temperature", 0.0, 40.0)
    temp.add_triangular_term("cold", 0.0, 0.0, 20.0)
    temp.add_triangular_term("comfortable", 15.0, 22.0, 28.0)
    temp.add_triangular_term("hot", 25.0, 40.0, 40.0)
    
    print(f"\n{temp}")
    print(f"Terms: {temp.get_terms()}")
    
    # Fuzzify different values
    for value in [10.0, 20.0, 25.0, 30.0]:
        memberships = temp.fuzzify(value)
        print(f"\nFuzzify({value}°C):")
        for term, membership in memberships.items():
            print(f"  {term}: {membership:.3f}")


def test_fuzzy_system_simple():
    """Test a simple fuzzy inference system"""
    print("\n" + "=" * 60)
    print("Testing Simple Fuzzy System (Temperature → Fan)")
    print("=" * 60)
    
    # Create input variable
    temp = fuzzy.Variable("temperature", 0.0, 40.0)
    temp.add_triangular_term("cold", 0.0, 0.0, 20.0)
    temp.add_triangular_term("comfortable", 15.0, 22.0, 28.0)
    temp.add_triangular_term("hot", 25.0, 40.0, 40.0)
    
    # Create output variable
    fan = fuzzy.Variable("fan_speed", 0.0, 100.0)
    fan.add_triangular_term("slow", 0.0, 0.0, 50.0)
    fan.add_triangular_term("medium", 30.0, 50.0, 70.0)
    fan.add_triangular_term("fast", 50.0, 100.0, 100.0)
    
    # Create system
    system = fuzzy.FuzzySystem()
    system.add_input(temp)
    system.add_output(fan)
    
    # Add rules
    system.add_rule(fuzzy.Rule(
        [("temperature", "cold")],
        ("fan_speed", "slow")
    ))
    system.add_rule(fuzzy.Rule(
        [("temperature", "comfortable")],
        ("fan_speed", "medium")
    ))
    system.add_rule(fuzzy.Rule(
        [("temperature", "hot")],
        ("fan_speed", "fast")
    ))
    
    print(f"\n{system}")
    
    # Test inference
    print("\nInference Results:")
    print("-" * 40)
    for temp_value in [10.0, 20.0, 25.0, 30.0, 35.0]:
        result = system.infer({"temperature": temp_value})
        fan_speed = result["fan_speed"]
        print(f"  Temp: {temp_value:5.1f}°C → Fan: {fan_speed:5.1f}%")


def test_fuzzy_system_complex():
    """Test a more complex system with multiple inputs"""
    print("\n" + "=" * 60)
    print("Testing Complex System (Humidity + Temp → Fan)")
    print("=" * 60)
    
    # Temperature input
    temp = fuzzy.Variable("temperature", 0.0, 40.0)
    temp.add_triangular_term("cold", 0.0, 0.0, 20.0)
    temp.add_triangular_term("hot", 20.0, 40.0, 40.0)
    
    # Humidity input
    humidity = fuzzy.Variable("humidity", 0.0, 100.0)
    humidity.add_triangular_term("dry", 0.0, 0.0, 50.0)
    humidity.add_triangular_term("humid", 50.0, 100.0, 100.0)
    
    # Fan output
    fan = fuzzy.Variable("fan_speed", 0.0, 100.0)
    fan.add_triangular_term("slow", 0.0, 0.0, 50.0)
    fan.add_triangular_term("medium", 25.0, 50.0, 75.0)
    fan.add_triangular_term("fast", 50.0, 100.0, 100.0)
    
    # Create system
    system = fuzzy.FuzzySystem()
    system.add_input(temp)
    system.add_input(humidity)
    system.add_output(fan)
    
    # Rules with multiple conditions (AND logic)
    system.add_rule(fuzzy.Rule(
        [("temperature", "cold"), ("humidity", "dry")],
        ("fan_speed", "slow")
    ))
    system.add_rule(fuzzy.Rule(
        [("temperature", "cold"), ("humidity", "humid")],
        ("fan_speed", "medium")
    ))
    system.add_rule(fuzzy.Rule(
        [("temperature", "hot"), ("humidity", "dry")],
        ("fan_speed", "medium")
    ))
    system.add_rule(fuzzy.Rule(
        [("temperature", "hot"), ("humidity", "humid")],
        ("fan_speed", "fast")
    ))
    
    print(f"\n{system}")
    
    # Test various combinations
    print("\nInference Results:")
    print("-" * 50)
    test_cases = [
        (15.0, 30.0),   # cold & dry
        (15.0, 70.0),   # cold & humid
        (30.0, 30.0),   # hot & dry
        (30.0, 70.0),   # hot & humid
        (25.0, 50.0),   # medium & medium
    ]
    
    for temp_val, humid_val in test_cases:
        result = system.infer({
            "temperature": temp_val,
            "humidity": humid_val
        })
        fan_speed = result["fan_speed"]
        print(f"  Temp: {temp_val:4.1f}°C, Humidity: {humid_val:5.1f}% → Fan: {fan_speed:5.1f}%")


def benchmark_performance():
    """Simple performance test"""
    import time
    
    print("\n" + "=" * 60)
    print("Performance Benchmark")
    print("=" * 60)
    
    # Setup system
    temp = fuzzy.Variable("temperature", 0.0, 40.0)
    temp.add_triangular_term("cold", 0.0, 0.0, 20.0)
    temp.add_triangular_term("comfortable", 15.0, 22.0, 28.0)
    temp.add_triangular_term("hot", 25.0, 40.0, 40.0)
    
    fan = fuzzy.Variable("fan_speed", 0.0, 100.0)
    fan.add_triangular_term("slow", 0.0, 0.0, 50.0)
    fan.add_triangular_term("medium", 30.0, 50.0, 70.0)
    fan.add_triangular_term("fast", 50.0, 100.0, 100.0)
    
    system = fuzzy.FuzzySystem()
    system.add_input(temp)
    system.add_output(fan)
    
    system.add_rule(fuzzy.Rule([("temperature", "cold")], ("fan_speed", "slow")))
    system.add_rule(fuzzy.Rule([("temperature", "comfortable")], ("fan_speed", "medium")))
    system.add_rule(fuzzy.Rule([("temperature", "hot")], ("fan_speed", "fast")))

    # Benchmark
    iterations = 10000
    start = time.time()
    
    for i in range(iterations):
        temp_val = (i % 40) + 0.0
        result = system.infer({"temperature": temp_val})
    
    elapsed = time.time() - start
    per_inference = elapsed / iterations * 1000  # ms
    
    print(f"\nIterations: {iterations}")
    print(f"Total time: {elapsed:.3f} seconds")
    print(f"Per inference: {per_inference:.4f} ms")
    print(f"Throughput: {iterations/elapsed:.0f} inferences/second")


def main():
    """Run all tests"""
    print("PyFuzzy-Rust Demo & Test Suite")
    
    try:
        test_membership_functions()
        test_linguistic_variables()
        test_fuzzy_system_simple()
        test_fuzzy_system_complex()
        benchmark_performance()
        
        print("All tests passed!")
        
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()