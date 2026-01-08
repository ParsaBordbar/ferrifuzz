PyFuzzy-Rust Demo & Test Suite
============================================================
Testing Membership Functions
============================================================

TriangularMF(0, 50, 100)
  membership(0) = 0.0
  membership(25) = 0.5
  membership(50) = 1.0
  membership(75) = 0.5
  membership(100) = 0.0

TrapezoidalMF(0, 20, 80, 100)
  membership(0) = 0.0
  membership(10) = 0.5
  membership(50) = 1.0
  membership(90) = 0.5
  membership(100) = 0.0

============================================================
Testing Linguistic Variables
============================================================

Variable('temperature', 0, 40, 3 terms)
Terms: ['cold', 'comfortable', 'hot']

Fuzzify(10.0°C):
  cold: 0.500
  comfortable: 0.000
  hot: 0.000

Fuzzify(20.0°C):
  cold: 0.000
  comfortable: 0.714
  hot: 0.000

Fuzzify(25.0°C):
  cold: 0.000
  comfortable: 0.500
  hot: 0.000

Fuzzify(30.0°C):
  cold: 0.000
  comfortable: 0.000
  hot: 0.333

============================================================
Testing Simple Fuzzy System (Temperature → Fan)
============================================================

FuzzySystem(1 inputs, 1 outputs, 3 rules)

Inference Results:
----------------------------------------
  Temp:  10.0°C → Fan:  19.7%
  Temp:  20.0°C → Fan:  50.0%
  Temp:  25.0°C → Fan:  50.0%
  Temp:  30.0°C → Fan:  78.6%
  Temp:  35.0°C → Fan:  81.7%

============================================================
Testing Complex System (Humidity + Temp → Fan)
============================================================

FuzzySystem(2 inputs, 1 outputs, 4 rules)

Inference Results:
--------------------------------------------------
  Temp: 15.0°C, Humidity:  30.0% → Fan:  22.3%
  Temp: 15.0°C, Humidity:  70.0% → Fan:  50.0%
  Temp: 30.0°C, Humidity:  30.0% → Fan:  50.0%
  Temp: 30.0°C, Humidity:  70.0% → Fan:  79.3%
  Temp: 25.0°C, Humidity:  50.0% → Fan:  50.0%

============================================================
Performance Benchmark
============================================================

Iterations: 10000
Total time: 0.036 seconds
Per inference: 0.0036 ms
Throughput: 280207 inferences/second
All tests passed!