/**
 * The property editors, by the type the model declares.
 *
 * A `.gmdl` property carries its type (`TSNA`, `PosInteger`, `EntityUse`,
 * `EntityDef`, `Listed`, `Expresion`, ...), and the editor renders one editor
 * per type instead of one free-text field for everything. What can be
 * checked without the engine is checked here, before the value reaches the
 * model.
 *
 * This module has no imports on purpose: it is the pure part of the editor,
 * and the tests run it without a bundler.
 */

/** The editor a declared type gets. */
export type PropertyEditor =
  /** A text field: a number, an expression or a name. */
  | "text"
  /** A number field. */
  | "number"
  /** A number field that only takes integers greater than zero. */
  | "integer"
  /** A number field that takes a percentage. */
  | "percent"
  /** A list of options. */
  | "select"
  /** A text field for an expression. */
  | "expression";

/** One declared property, as the editor needs it. */
export interface PropertySpec {
  /** Property name (`Media`, `Queue`, ...). */
  name: string;
  /** Declared type (`TSNA`, `EntityUse`, `Listed`, ...). */
  ty: string;
  /** Declared subtype (`SNA`, `Queue`, ...). */
  subtype: string;
  /** The options a `Listed` property offers. */
  options: string[];
}

/**
 * The editor a property gets, from the type it declares.
 *
 * The type decides the editor and the validation, not the name of the
 * property: two properties called `Media` in two different classes are two
 * different things when their types differ.
 */
export function editorFor(property: PropertySpec): PropertyEditor {
  switch (property.ty) {
    case "PosInteger":
      return "integer";
    case "FloatPercent":
      return "percent";
    case "FloatPercentList":
      return "text";
    case "Listed":
      return "select";
    case "Expresion":
      return "expression";
    case "TSNA":
      return "number";
    case "EntityUse":
    case "EntityDef":
      return "text";
    default:
      return "text";
  }
}

/**
 * Why a value cannot reach the model, in the words of the interface.
 *
 * `null` means the value is acceptable. An empty value is acceptable for
 * every type except the ones that need a number: the engine reads an empty
 * number as zero and the original editor left the field empty too.
 */
export function validateProperty(property: PropertySpec, value: string): string | null {
  const text = value.trim();
  const editor = editorFor(property);
  switch (editor) {
    case "integer": {
      if (text === "") {
        return null;
      }
      if (!/^\d+$/.test(text)) {
        return "El valor debe ser un número entero.";
      }
      if (Number(text) < 1) {
        return "El valor debe ser mayor que cero.";
      }
      return null;
    }
    case "number":
    case "percent": {
      if (text === "") {
        return null;
      }
      if (!isNumber(text)) {
        return "El valor debe ser un número.";
      }
      return null;
    }
    case "select": {
      if (text === "") {
        return null;
      }
      if (property.options.length > 0 && !property.options.includes(text)) {
        return "Elegí una de las opciones.";
      }
      return null;
    }
    case "expression": {
      if (text === "") {
        return "El valor no puede estar vacío.";
      }
      return null;
    }
    case "text":
    default: {
      if (property.ty === "EntityDef") {
        // The engine refuses a definition name with whitespace or a colon.
        if (text !== "" && /\s/.test(text)) {
          return "El nombre no puede tener espacios.";
        }
        if (text !== "" && text.includes(":")) {
          return "El nombre no puede contener «:».";
        }
      }
      if (property.ty === "EntityUse" && text !== "" && /\s/.test(text)) {
        return "El nombre no puede tener espacios.";
      }
      return null;
    }
  }
}

/** Whether a text is a number. */
function isNumber(text: string): boolean {
  const parsed = Number(text);
  return Number.isFinite(parsed);
}
