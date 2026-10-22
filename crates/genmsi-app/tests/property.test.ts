import assert from "node:assert/strict";
import { test } from "node:test";
import { editorFor, validateProperty, type PropertySpec } from "../src/edit/property.ts";

/** A declared property, as the model carries it. */
function property(
  name: string,
  ty: string,
  subtype = "",
  options: string[] = [],
): PropertySpec {
  return { name, ty, subtype, options };
}

test("the declared type decides the editor", () => {
  assert.equal(editorFor(property("Media", "TSNA", "SNA")), "number");
  assert.equal(editorFor(property("Semilla", "PosInteger")), "integer");
  assert.equal(editorFor(property("Queue", "EntityUse", "Queue")), "text");
  assert.equal(editorFor(property("NombreFunc", "EntityDef", "Function")), "text");
  assert.equal(editorFor(property("Operador", "Listed", "op_AddSub")), "select");
  assert.equal(editorFor(property("Expresion", "Expresion", "Expresion")), "expression");
  assert.equal(editorFor(property("Prob", "FloatPercent")), "percent");
});

test("a positive integer is checked before it reaches the model", () => {
  const spec = property("Semilla", "PosInteger");
  assert.equal(validateProperty(spec, "7"), null);
  assert.ok(validateProperty(spec, "0") !== null);
  assert.ok(validateProperty(spec, "-3") !== null);
  assert.ok(validateProperty(spec, "dos") !== null);
});

test("a number is checked before it reaches the model", () => {
  const spec = property("Media", "TSNA", "SNA");
  assert.equal(validateProperty(spec, "60"), null);
  assert.equal(validateProperty(spec, "60.5"), null);
  assert.ok(validateProperty(spec, "sesenta") !== null);
});

test("a definition name cannot have spaces or a colon", () => {
  const spec = property("NombreFunc", "EntityDef", "Function");
  assert.equal(validateProperty(spec, "MediaClientes"), null);
  assert.ok(validateProperty(spec, "tiempo demora") !== null);
  assert.ok(validateProperty(spec, "Q:cola") !== null);
});

test("an option of a listed property must be one of its options", () => {
  const spec = property("Operador", "Listed", "op_AddSub", ["+", "-", "="]);
  assert.equal(validateProperty(spec, "+"), null);
  assert.ok(validateProperty(spec, "*") !== null);
});

test("an expression cannot be empty", () => {
  const spec = property("Expresion", "Expresion", "Expresion");
  assert.ok(validateProperty(spec, "") !== null);
  assert.equal(validateProperty(spec, "FN:MediaClientes"), null);
});
