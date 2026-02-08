//! Test WebAssembly module for WasmEdge Rust SDK integration tests
//!
//! This module contains various functions to test different WebAssembly features
//! when compiled to WASM and run in WasmEdge.

// ============================================================================
// Basic Arithmetic Operations
// ============================================================================

/// Add two i32 numbers
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two i32 numbers
#[no_mangle]
pub extern "C" fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiply two i32 numbers
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divide two i32 numbers (integer division)
#[no_mangle]
pub extern "C" fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0; // Avoid division by zero
    }
    a / b
}

/// Modulo operation
#[no_mangle]
pub extern "C" fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;
    }
    a % b
}

// ============================================================================
// 64-bit Integer Operations
// ============================================================================

/// Add two i64 numbers
#[no_mangle]
pub extern "C" fn add_i64(a: i64, b: i64) -> i64 {
    a + b
}

/// Multiply two i64 numbers
#[no_mangle]
pub extern "C" fn multiply_i64(a: i64, b: i64) -> i64 {
    a * b
}

// ============================================================================
// Floating Point Operations
// ============================================================================

/// Add two f32 numbers
#[no_mangle]
pub extern "C" fn add_f32(a: f32, b: f32) -> f32 {
    a + b
}

/// Multiply two f32 numbers
#[no_mangle]
pub extern "C" fn multiply_f32(a: f32, b: f32) -> f32 {
    a * b
}

/// Add two f64 numbers
#[no_mangle]
pub extern "C" fn add_f64(a: f64, b: f64) -> f64 {
    a + b
}

/// Multiply two f64 numbers
#[no_mangle]
pub extern "C" fn multiply_f64(a: f64, b: f64) -> f64 {
    a * b
}

/// Calculate square root of f64
#[no_mangle]
pub extern "C" fn sqrt_f64(x: f64) -> f64 {
    x.sqrt()
}

/// Calculate power of f64
#[no_mangle]
pub extern "C" fn pow_f64(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

// ============================================================================
// Recursive Functions
// ============================================================================

/// Calculate factorial recursively
#[no_mangle]
pub extern "C" fn factorial(n: i32) -> i64 {
    if n <= 1 {
        1
    } else {
        n as i64 * factorial(n - 1)
    }
}

/// Calculate fibonacci number recursively
#[no_mangle]
pub extern "C" fn fibonacci(n: i32) -> i64 {
    if n <= 0 {
        0
    } else if n == 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

/// Calculate fibonacci iteratively (more efficient)
#[no_mangle]
pub extern "C" fn fibonacci_iter(n: i32) -> i64 {
    if n <= 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut a: i64 = 0;
    let mut b: i64 = 1;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

// ============================================================================
// Bitwise Operations
// ============================================================================

/// Bitwise AND
#[no_mangle]
pub extern "C" fn bitwise_and(a: i32, b: i32) -> i32 {
    a & b
}

/// Bitwise OR
#[no_mangle]
pub extern "C" fn bitwise_or(a: i32, b: i32) -> i32 {
    a | b
}

/// Bitwise XOR
#[no_mangle]
pub extern "C" fn bitwise_xor(a: i32, b: i32) -> i32 {
    a ^ b
}

/// Bitwise NOT
#[no_mangle]
pub extern "C" fn bitwise_not(a: i32) -> i32 {
    !a
}

/// Left shift
#[no_mangle]
pub extern "C" fn shift_left(a: i32, bits: i32) -> i32 {
    a << bits
}

/// Right shift (arithmetic)
#[no_mangle]
pub extern "C" fn shift_right(a: i32, bits: i32) -> i32 {
    a >> bits
}

// ============================================================================
// Comparison Operations
// ============================================================================

/// Check if a > b
#[no_mangle]
pub extern "C" fn greater_than(a: i32, b: i32) -> i32 {
    if a > b { 1 } else { 0 }
}

/// Check if a < b
#[no_mangle]
pub extern "C" fn less_than(a: i32, b: i32) -> i32 {
    if a < b { 1 } else { 0 }
}

/// Check if a == b
#[no_mangle]
pub extern "C" fn equals(a: i32, b: i32) -> i32 {
    if a == b { 1 } else { 0 }
}

/// Return maximum of two values
#[no_mangle]
pub extern "C" fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

/// Return minimum of two values
#[no_mangle]
pub extern "C" fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

/// Clamp value between min and max
#[no_mangle]
pub extern "C" fn clamp(value: i32, min_val: i32, max_val: i32) -> i32 {
    if value < min_val {
        min_val
    } else if value > max_val {
        max_val
    } else {
        value
    }
}

// ============================================================================
// Mathematical Functions
// ============================================================================

/// Calculate absolute value
#[no_mangle]
pub extern "C" fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

/// Calculate absolute value for f64
#[no_mangle]
pub extern "C" fn abs_f64(x: f64) -> f64 {
    x.abs()
}

/// Check if number is prime
#[no_mangle]
pub extern "C" fn is_prime(n: i32) -> i32 {
    if n <= 1 {
        return 0;
    }
    if n <= 3 {
        return 1;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return 0;
    }

    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return 0;
        }
        i += 6;
    }
    1
}

/// Calculate GCD using Euclidean algorithm
#[no_mangle]
pub extern "C" fn gcd(mut a: i32, mut b: i32) -> i32 {
    a = abs(a);
    b = abs(b);
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Calculate LCM
#[no_mangle]
pub extern "C" fn lcm(a: i32, b: i32) -> i32 {
    if a == 0 || b == 0 {
        return 0;
    }
    abs(a) / gcd(a, b) * abs(b)
}

/// Calculate sum of digits
#[no_mangle]
pub extern "C" fn sum_of_digits(mut n: i32) -> i32 {
    n = abs(n);
    let mut sum = 0;
    while n > 0 {
        sum += n % 10;
        n /= 10;
    }
    sum
}

/// Count number of digits
#[no_mangle]
pub extern "C" fn count_digits(mut n: i32) -> i32 {
    if n == 0 {
        return 1;
    }
    n = abs(n);
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

/// Reverse digits of a number
#[no_mangle]
pub extern "C" fn reverse_number(mut n: i32) -> i32 {
    let negative = n < 0;
    n = abs(n);
    let mut reversed = 0;
    while n > 0 {
        reversed = reversed * 10 + n % 10;
        n /= 10;
    }
    if negative { -reversed } else { reversed }
}

/// Check if number is palindrome
#[no_mangle]
pub extern "C" fn is_palindrome(n: i32) -> i32 {
    if n < 0 {
        return 0;
    }
    if n == reverse_number(n) { 1 } else { 0 }
}

// ============================================================================
// Array/Loop Operations (using linear memory implicitly)
// ============================================================================

/// Calculate sum of first n natural numbers
#[no_mangle]
pub extern "C" fn sum_natural(n: i32) -> i64 {
    let n = n as i64;
    n * (n + 1) / 2
}

/// Calculate sum of squares of first n natural numbers
#[no_mangle]
pub extern "C" fn sum_squares(n: i32) -> i64 {
    let n = n as i64;
    n * (n + 1) * (2 * n + 1) / 6
}

/// Calculate sum of cubes of first n natural numbers
#[no_mangle]
pub extern "C" fn sum_cubes(n: i32) -> i64 {
    let sum = sum_natural(n as i32);
    sum * sum
}

/// Calculate n-th triangular number
#[no_mangle]
pub extern "C" fn triangular_number(n: i32) -> i64 {
    sum_natural(n)
}

/// Calculate n-th power of 2
#[no_mangle]
pub extern "C" fn power_of_two(n: i32) -> i64 {
    if n < 0 || n > 62 {
        return 0;
    }
    1_i64 << n
}

// ============================================================================
// Type Conversion Functions
// ============================================================================

/// Convert i32 to i64
#[no_mangle]
pub extern "C" fn i32_to_i64(x: i32) -> i64 {
    x as i64
}

/// Convert i64 to i32 (truncate)
#[no_mangle]
pub extern "C" fn i64_to_i32(x: i64) -> i32 {
    x as i32
}

/// Convert f32 to i32 (truncate)
#[no_mangle]
pub extern "C" fn f32_to_i32(x: f32) -> i32 {
    x as i32
}

/// Convert i32 to f32
#[no_mangle]
pub extern "C" fn i32_to_f32(x: i32) -> f32 {
    x as f32
}

/// Convert f64 to i64 (truncate)
#[no_mangle]
pub extern "C" fn f64_to_i64(x: f64) -> i64 {
    x as i64
}

/// Convert i64 to f64
#[no_mangle]
pub extern "C" fn i64_to_f64(x: i64) -> f64 {
    x as f64
}

// ============================================================================
// Complex Calculations
// ============================================================================

/// Calculate compound interest: principal * (1 + rate)^time
#[no_mangle]
pub extern "C" fn compound_interest(principal: f64, rate: f64, time: i32) -> f64 {
    principal * (1.0 + rate).powi(time)
}

/// Calculate distance between two 2D points
#[no_mangle]
pub extern "C" fn distance_2d(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

/// Calculate area of circle given radius
#[no_mangle]
pub extern "C" fn circle_area(radius: f64) -> f64 {
    core::f64::consts::PI * radius * radius
}

/// Calculate circumference of circle given radius
#[no_mangle]
pub extern "C" fn circle_circumference(radius: f64) -> f64 {
    2.0 * core::f64::consts::PI * radius
}

/// Calculate hypotenuse given two sides
#[no_mangle]
pub extern "C" fn hypotenuse(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

/// Solve quadratic equation ax^2 + bx + c = 0, return discriminant
#[no_mangle]
pub extern "C" fn quadratic_discriminant(a: f64, b: f64, c: f64) -> f64 {
    b * b - 4.0 * a * c
}
