<?php

namespace MultiAssign;

use function PHPStan\Testing\assertType;

class Foo
{
	public function fooMethod(): void {}
}

class Bar
{
	public function barMethod(): void {}
}

// Chain assignments ($a = $b = expr) are not yet resolved by PHPantom's
// variable resolution pipeline. All assertions below are SKIP until
// chain assignment support is implemented.

function multiAssignNull(): void {
	$foo = $bar = $baz = null;
	assertType('null', $foo);
	assertType('null', $bar);
	assertType('null', $baz);
}

function multiAssignInt(): void {
	$a = $b = $c = 42;
	assertType('int', $a);
	assertType('int', $b);
	assertType('int', $c);
}

function multiAssignString(): void {
	$a = $b = 'hello';
	assertType('string', $a);
	assertType('string', $b);
}

function multiAssignFloat(): void {
	$a = $b = 3.14;
	assertType('float', $a);
	assertType('float', $b);
}

function multiAssignBool(): void {
	$a = $b = true;
	assertType('bool', $a);
	assertType('bool', $b);
}

function multiAssignObject(): void {
	$a = $b = new Foo();
	assertType('Foo', $a);
	assertType('Foo', $b);
}

function multiAssignFromParam(int $x): void {
	$a = $b = $x;
	assertType('int', $a);
	assertType('int', $b);
}

/**
 * @param Foo|Bar $union
 */
function multiAssignUnion($union): void {
	$a = $b = $union;
	assertType('Foo|Bar', $a);
	assertType('Foo|Bar', $b);
}

function reassignAfterChain(): void {
	$a = $b = 1;
	assertType('int', $a);
	assertType('int', $b);

	$a = 'changed';
	assertType('string', $a);
	assertType('int', $b);
}

function multiAssignArray(): void {
	$a = $b = [1, 2, 3];
	assertType('list<int>', $a);
	assertType('list<int>', $b);
}

/**
 * @param string|null $nullable
 */
function multiAssignNullable(?string $nullable): void {
	$a = $b = $nullable;
	assertType('string|null', $a);
	assertType('string|null', $b);
}