<?php
// Source: Psalm BinaryOperationTest.php
// Auto-extracted by scripts/extract_psalm_tests.php
// Do not edit manually — re-run the extraction script instead.

// Test: regularAddition
namespace PsalmTest_binary_operation_1 {
    $a = 5 + 4;

    assertType('int', $a); // SKIP literal arithmetic resolves to int|float instead of int
}

// Test: differingNumericTypesAdditionInWeakMode
namespace PsalmTest_binary_operation_2 {
    $a = 5 + 4.1;

    assertType('float', $a); // SKIP literal arithmetic resolves to int|float instead of float
}

// Test: modulo
namespace PsalmTest_binary_operation_3 {
    $a = 25 % 2;
    $b = 25.4 % 2;
    $c = 25 % 2.5;
    $d = 25.5 % 2.5;
    $e = 25 % 1;

    assertType('int', $a);
    assertType('int', $b);
    assertType('int', $c);
    assertType('int', $d);
    assertType('int', $e);
}

// Test: concatenationWithTwoLiteralInt
namespace PsalmTest_binary_operation_4 {
    $a = 7 . 5;

    assertType('string', $a);
}

// Test: bitwiseoperations
namespace PsalmTest_binary_operation_5 {
    $a = 4 & 5;
    $b = 2 | 3;
    $c = 4 ^ 3;
    $d = 1 << 2;
    $e = 15 >> 2;
    $f = "a" & "b";

    assertType('int', $a);
    assertType('int', $b);
    assertType('int', $c);
    assertType('int', $d);
    assertType('int', $e);
    assertType('string', $f); // SKIP bitwise op on strings resolves to int instead of string
}

// Test: booleanXor
namespace PsalmTest_binary_operation_6 {
    $a = 4 ^ 1;
    $b = 3 ^ 1;
    $c = (true xor false);
    $d = (false xor false);

    assertType('int', $a);
    assertType('int', $b);
    assertType('bool', $c); // SKIP xor on bools resolves to int instead of bool
    assertType('bool', $d); // SKIP xor on bools resolves to int instead of bool
}

// Test: floatIncrement
namespace PsalmTest_binary_operation_7 {
    $a = 1.1;
    $a++;
    $b = 1.1;
    $b += 1;

    assertType('float', $a);
    assertType('float', $b);
}

// Test: exponent
namespace PsalmTest_binary_operation_8 {
    $b = 4 ** 5;

    assertType('int', $b); // SKIP exponent resolves to int|float instead of int
}

// Test: bitwiseNot
namespace PsalmTest_binary_operation_9 {
    $a = ~4;
    $b = ~4.0;
    $c = ~4.4;
    $d = ~"a";

    assertType('int', $a);
    assertType('int', $b);
    assertType('int', $c);
    assertType('string', $d); // SKIP bitwise not on string resolves to int instead of string
}

// Test: stringIncrementSuppressed
namespace PsalmTest_binary_operation_10 {
    $a = "hello";
    /** @psalm-suppress StringIncrement */
    $a++;

    assertType('string', $a);
}

// Test: numericWithInt
namespace PsalmTest_binary_operation_16 {
    /** @return numeric */
    function getNumeric(){
        return 1;
    }
    $a = getNumeric();
    $a++;
    $b = getNumeric() * 2;
    $c = 1 - getNumeric();
    $d = 2;
    $d -= getNumeric();

    assertType('float|int', $a); // SKIP numeric increment resolves to string instead of float|int
    assertType('float|int', $b);
    assertType('float|int', $c);
    assertType('float|int', $d); // SKIP compound assignment with numeric type resolves to int
}

// Test: NumericStringIncrementLiteral
namespace PsalmTest_binary_operation_17 {
    $a = "123";
    $b = "123";
    $a++;
    ++$b;

    assertType('float|int', $a); // SKIP string increment resolves to string instead of float|int
    assertType('float|int', $b); // SKIP string increment resolves to string instead of float|int
}

