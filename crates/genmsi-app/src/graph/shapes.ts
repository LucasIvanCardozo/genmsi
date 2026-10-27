/**
 * The visual shape of every block class.
 *
 * A block of the model is not a rounded rectangle any more: every class of
 * the catalogue has its own silhouette, so the reader recognises a block by
 * its shape before reading its name (S2). This module is the single source
 * of that shape (S6): the node of the canvas and the thumbnail of the
 * catalogue draw the same silhouette, or the list of blocks lies.
 *
 * The module holds, for every class:
 *
 * - the **family** it belongs to (the language of shapes) and the name of
 *   its **variant** inside that family (S2, §5.1 of the contract);
 * - the **footprint** the class is drawn at (S7: the silhouette is drawn
 *   inside the footprint the model already carries);
 * - the **path** of the silhouette and the **details** that stroke its
 *   inside;
 * - the **ports**, as fractions of the footprint, at the anchor the
 *   original editor put them (S8);
 * - the **label** zone and the **figure** zone, in footprint pixels, which
 *   never overlap each other (S9).
 *
 * It also holds the three **colour levels** of every family: the strong
 * tone (`acento`) for the live figure and the dot of the port, the same
 * tone lightened (`borde`) for the outline of the silhouette, and the same
 * tone lightened a lot (`relleno`) for the fill of the block.
 *
 * The module is pure data plus small pure helpers: it imports nothing, so
 * it can be unit tested with `node --test`.
 *
 * GENERATED FILE: do not edit it by hand. Its source of truth is
 * `odd/tasks/forma-bloques-diseno/generar_shapes.py`, which reads the
 * approved design in `odd/tasks/forma-bloques-diseno/grupos/*.md`.
 */

/** The family of a block: the language of shapes it belongs to. */
export type Family =
  | "Fuente"
  | "Destino"
  | "Decisión"
  | "Cola"
  | "Recurso"
  | "Depósito"
  | "Tabla"
  | "Cadena"
  | "Grupo"
  | "Selección"
  | "Operación"
  | "Anotación"
;

/**
 * The three colour levels of a family.
 *
 * The three levels are the same tone at three strengths: the family is
 * recognised by the hue, and the level says what the colour is for.
 */
export interface FamilyColors {
  /** The strong tone: the live figure and the dot of the port. */
  acento: string;
  /** The same tone, lightened: the outline of the silhouette. */
  borde: string;
  /** The same tone, lightened a lot: the fill of the block. */
  relleno: string;
}

/** Where a port of a block sits, as a fraction of the footprint. */
export interface PortAnchor {
  /** The name of the port, as the catalogue declares it. */
  name: string;
  /** The horizontal anchor: 0 is the left border, 1 the right one. */
  x: number;
  /** The vertical anchor: 0 is the top border, 1 the bottom one. */
  y: number;
}

/** A rectangular zone inside the footprint of a block. */
export interface Zone {
  /** The left of the zone, in footprint pixels. */
  x: number;
  /** The top of the zone, in footprint pixels. */
  y: number;
  /** The width of the zone, in footprint pixels. */
  width: number;
  /** The height of the zone, in footprint pixels. */
  height: number;
}

/** The shape of one class of block. */
export interface BlockShape {
  /** The family the class belongs to. */
  family: Family;
  /** The name of the silhouette inside the family. */
  variant: string;
  /** The size the class is drawn at. */
  footprint: { width: number; height: number };
  /** The `d` of the silhouette, in the coordinates of the footprint. */
  path: string;
  /** The `d` of the interior strokes, drawn without fill. */
  details: string[];
  /** Where the ports sit, as fractions of the footprint. */
  ports: PortAnchor[];
  /** The zone of the label and of the subtitle. */
  label: Zone;
  /** The zone of the live figure, or `null` when the class has none. */
  figure: Zone | null;
  /** Why the class has this silhouette, in the words of the design. */
  note: string;
}

/** The three colour levels of every family. */
export const FAMILY_COLORS: Record<Family, FamilyColors> = {
  "Fuente": { acento: "#047857", borde: "#b3c4bc", relleno: "#f3f6f5" },
  "Destino": { acento: "#be123c", borde: "#deb3b7", relleno: "#f9f4f4" },
  "Decisión": { acento: "#4338ca", borde: "#b8b7e4", relleno: "#f4f4fa" },
  "Cola": { acento: "#b45309", borde: "#dabbb3", relleno: "#f9f4f3" },
  "Recurso": { acento: "#0f766e", borde: "#b3c4c1", relleno: "#f4f6f5" },
  "Depósito": { acento: "#0369a1", borde: "#b3c0d2", relleno: "#f3f5f8" },
  "Tabla": { acento: "#6d28d9", borde: "#c1b5eb", relleno: "#f5f4fc" },
  "Cadena": { acento: "#c2410c", borde: "#e0b8b3", relleno: "#faf4f3" },
  "Grupo": { acento: "#be185d", borde: "#deb4bd", relleno: "#f9f4f5" },
  "Selección": { acento: "#0e7490", borde: "#b3c3cc", relleno: "#f3f6f7" },
  "Operación": { acento: "#475569", borde: "#b9bcc0", relleno: "#f4f5f5" },
  "Anotación": { acento: "#64748b", borde: "#bfc3ca", relleno: "#f5f6f6" },
};

/** The shape of every class of the catalogue, by class name. */
export const BLOCK_SHAPES: Record<string, BlockShape> = {
  "Clock": {
    family: "Fuente",
    variant: "esfera del reloj",
    footprint: { width: 50, height: 50 },
    path: "M 0 25 A 25 25 0 1 0 50 25 A 25 25 0 1 0 0 25 Z",
    details: [
      "M 2 25 L 6 25",
      "M 44 25 L 48 25",
    ],
    ports: [],
    label: { x: 7, y: 12, width: 36, height: 18 },
    figure: { x: 10, y: 32, width: 30, height: 12 },
    note: "esfera de la familia Fuente en placa apagada, con dos marcas de esfera a los lados; sin puertos no hay ancla, y al medir 50 px de ancho útil (38 px) el rótulo va arriba y la zona de cifra reservada abajo y vacía, porque es una declaración sin entidad",
  },
  "Generate": {
    family: "Fuente",
    variant: "caja de emisión",
    footprint: { width: 120, height: 63 },
    path: "M 31.5 0 A 31.5 31.5 0 0 1 57.83 20 L 120 20 L 120 63 L 31.5 63 A 31.5 31.5 0 0 1 31.5 0 Z",
    details: [
      "M 62 20 L 62 63",
    ],
    ports: [
      { name: "Salida1", x: 0.27, y: 1 },
    ],
    label: { x: 10, y: 26, width: 48, height: 24 },
    figure: { x: 64, y: 32, width: 32, height: 14 },
    note: "cabeza redonda unida a la caja de emisión, la silueta del catálogo original; la caja sube a y=20 para que rótulo y cifra compartan banda, y el único puerto sale por abajo en el 27% del ancho, sobre el borde plano de la caja",
  },
  "Terminate": {
    family: "Destino",
    variant: "circulo limpio",
    footprint: { width: 64, height: 64 },
    path: "M32 0 A32 32 0 1 1 32 64 A32 32 0 1 1 32 0 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
    ],
    label: { x: 5, y: 16, width: 54, height: 11 },
    figure: { x: 14, y: 34, width: 36, height: 11 },
    note: "Circulo limpio, la forma de sumidero que se lee de un vistazo: el flujo entra por arriba y se cierra. El editor original lo dibujaba como una elipse de 64 con un rectangulo pegado al lado; esa pieza es adorno y no se lleva, porque hacia la silueta irregular y el anillo del fondo no significaba nada para quien lee. El puerto de entrada conserva su ancla exacta del original (32,0), que en un circulo es justo el vertice superior; por eso la huella de la clase pasa de 84x64 a 64x64. El rotulo y la cifra van en bandas apiladas porque el ancho util del circulo no da para los dos en el mismo lado. La zona de rotulo se ensancha a 54 px: la caja inscrita del circulo da de sobra a esa altura, y «Terminate» (48,6 px) cabe entero en una linea.",
  },
  "GateF": {
    family: "Decisión",
    variant: "rombo con accesorio",
    footprint: { width: 112, height: 48 },
    path: "M0 24 L38 0 L76 24 L38 48 Z M76 24 L94 12 L112 24 L94 36 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.33, y: 0 },
      { name: "Salida1", x: 0.33, y: 1 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 18, y: 13, width: 40, height: 12 },
    figure: { x: 21, y: 26, width: 34, height: 11 },
    note: "Variante «rombo con accesorio» de la familia Decisión: un rombo descentrado (vértices en 0,24 y 76,24) más el accesorio que dibuja la entidad que la compuerta vigila (aquí un rombo, el recurso); el accesorio es la firma de la clase y no lleva texto, porque el rombo mide 76 px y su caja inscrita (38 px de ancho) es menor de 60: el rótulo va en la banda superior y la cifra en la inferior, en la misma columna; las anclas (38,0), (38,48) y (0,24) del catálogo caen en los vértices.",
  },
  "GateL": {
    family: "Decisión",
    variant: "rombo con accesorio",
    footprint: { width: 102, height: 48 },
    path: "M0 24 L38 0 L76 24 L38 48 Z M76 12 H102 V38 H76 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.37, y: 0 },
      { name: "Salida1", x: 0.37, y: 1 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 18, y: 13, width: 40, height: 12 },
    figure: { x: 21, y: 26, width: 34, height: 11 },
    note: "Variante «rombo con accesorio»: el accesorio es una caja (el interruptor que la compuerta consulta) y por eso la compuerta se distingue de GateF, GateM y GateS de un vistazo; el rombo lleva el rótulo arriba y la cifra abajo porque su caja inscrita es de 38 px de ancho; anclas del catálogo en los vértices: (38,0) entrada, (38,48) salida de abajo y (0,24) salida lateral.",
  },
  "GateM": {
    family: "Decisión",
    variant: "rombo con accesorio",
    footprint: { width: 102, height: 48 },
    path: "M0 24 L38 0 L76 24 L38 48 Z M76 12 H102 V38 L89 32 L76 38 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.37, y: 0 },
      { name: "Salida1", x: 0.37, y: 1 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 18, y: 13, width: 40, height: 12 },
    figure: { x: 21, y: 26, width: 34, height: 11 },
    note: "Variante «rombo con accesorio»: el accesorio es una caja con la muesca que declara el modo de comparación, y la muesca sube por la izquierda como en el original; el rombo lleva el rótulo arriba y la cifra abajo por su caja inscrita de 38 px, y la salida lateral sigue siendo el vértice izquierdo (0,24).",
  },
  "GateS": {
    family: "Decisión",
    variant: "rombo con accesorio",
    footprint: { width: 102, height: 48 },
    path: "M0 24 L38 0 L76 24 L38 48 Z M76 25 A13 13 0 1 1 102 25 A13 13 0 1 1 76 25 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.37, y: 0 },
      { name: "Salida1", x: 0.37, y: 1 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 18, y: 13, width: 40, height: 12 },
    figure: { x: 21, y: 26, width: 34, height: 11 },
    note: "Variante «rombo con accesorio»: el accesorio es una elipse (el depósito que la compuerta vigila), el mismo lenguaje de burbuja que usan Join y Remove; el accesorio queda vacío de texto y el rombo lleva el rótulo arriba y la cifra abajo; las anclas (38,0), (38,48) y (0,24) son las del catálogo y caen en vértices del rombo.",
  },
  "Loop": {
    family: "Decisión",
    variant: "cuerpo con punta a la izquierda",
    footprint: { width: 96, height: 32 },
    path: "M0 16 L18 0 H88 L96 8 V24 L88 32 H18 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.59, y: 0 },
      { name: "Salida1", x: 0.59, y: 1 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 19, y: 5, width: 32, height: 12 },
    figure: { x: 19, y: 18, width: 32, height: 11 },
    note: "Variante «cuerpo con punta a la izquierda» de la familia Decisión: la punta ES la salida de retroceso (0,16) del original, y la entrada y la salida de paso cruzan el borde superior e inferior en x=57, que no es el centro del cuerpo (18-96), por eso el cuerpo se desplaza a la derecha; esa columna de entrada/salida parte el cuerpo en dos tramos de 33 px, ninguno de 60, así que el rótulo va arriba y la cifra abajo en el tramo izquierdo.",
  },
  "Multifurcacion": {
    family: "Decisión",
    variant: "peine de la multifurcacion",
    footprint: { width: 80, height: 40 },
    path: "M2 0 H78 L80 40 L70 30 L60 40 L50 30 L40 40 L30 30 L20 40 L10 30 L0 40 Z",
    details: [],
    ports: [
      { name: "Entrada", x: 0.5, y: 0 },
      { name: "Salida1", x: 0, y: 1 },
      { name: "SalidaProb2", x: 0.25, y: 1 },
      { name: "SalidaProb3", x: 0.5, y: 1 },
      { name: "SalidaProb4", x: 0.75, y: 1 },
      { name: "SalidaProb5", x: 1, y: 1 },
    ],
    label: { x: 2, y: 6, width: 48, height: 24 },
    figure: { x: 50, y: 9, width: 26, height: 10 },
    note: "Variante «peine de la multifurcacion» de la familia Decisión: los cinco dientes son exactamente las cinco salidas (0,32), (20,32), (40,32), (60,32) y (80,32) y la entrada sigue en el borde superior (40,0); la clase es de 20 px de alto en el catálogo, pero el peine necesita sitio para sus dos bandas de texto, así que la entrega usa una huella dibujada de 80×40 (la app dibuja el nodo con 32 px de alto mínimo, y aquí hacen falta 40 para que el rótulo y la cifra no se pisen); el texto va en banda propia sobre la corona (rótulo arriba, cifra abajo) porque el nombre completo (64 px) más la cifra (27 px) no caben de lado en 80 px, y las dos bandas caben en la franja superior de 24 px que deja la corona; el nombre ya no llega truncado. «Multifurcacion» no cabe en una linea de la corona, asi que va en dos («Multifu» y «rcacion»): la zona de rotulo ocupa 48 px y 24 px de alto sobre la corona, y la cifra se queda a su derecha con 26 px.",
  },
  "MultifurcacionP": {
    family: "Decisión",
    variant: "peine de la multifurcacion",
    footprint: { width: 80, height: 40 },
    path: "M2 0 H78 L80 40 L70 30 L60 40 L50 30 L40 40 L30 30 L20 40 L10 30 L0 40 Z",
    details: [
      "M56 4.5 H72",
      "M64.5 4.5 A2.5 2.5 0 1 1 59.5 4.5 A2.5 2.5 0 1 1 64.5 4.5 Z",
    ],
    ports: [
      { name: "Entrada", x: 0.5, y: 0 },
      { name: "Salida1", x: 0, y: 1 },
      { name: "SalidaProb2", x: 0.25, y: 1 },
      { name: "SalidaProb3", x: 0.5, y: 1 },
      { name: "SalidaProb4", x: 0.75, y: 1 },
      { name: "SalidaProb5", x: 1, y: 1 },
    ],
    label: { x: 2, y: 6, width: 48, height: 24 },
    figure: { x: 50, y: 9, width: 26, height: 10 },
    note: "Variante «peine de la multifurcacion», la misma silueta de la otra multifurcación: lo que la distingue es el raíl y la perilla del parámetro sobre la corona, y no un nombre distinto; las cinco salidas y la entrada tienen las mismas anclas del catálogo, y también se entrega sobre la huella dibujada de 80×40 (la clase es de 20 px), con el texto en banda propia sobre la corona por lo mismo que en la otra multifurcación, y el raíl y la perilla aparte, en la esquina superior izquierda, sobre la banda, para que no se pisen con el rótulo. «MultifurcacionP» va en dos lineas («Multifu» y «rcacionP»): la zona de rotulo ocupa 48 px y 24 px de alto sobre la corona, la cifra se queda a su derecha con 26 px, y el raíl y la perilla del parametro quedan en la esquina, sobre la banda.",
  },
  "Split": {
    family: "Decisión",
    variant: "horquilla del split",
    footprint: { width: 88, height: 32 },
    path: "M0 0 H88 V32 H52 L44 22 L36 32 H0 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0, y: 1 },
      { name: "Salida2", x: 1, y: 1 },
    ],
    label: { x: 6, y: 7, width: 38, height: 12 },
    figure: { x: 48, y: 8, width: 30, height: 11 },
    note: "Variante «horquilla del split» de la familia Decisión: el borde inferior se abre en una horquilla y las dos esquinas de abajo son las dos salidas (0,32) y (88,32), con la entrada en el borde superior (44,0); la horquilla se hunde sólo hasta y=22 para que la banda de texto (76 px, más de 60) quepa entera, porque en las punteras no cabe una zona de cifra de 30 px.",
  },
  "Test": {
    family: "Decisión",
    variant: "rombo",
    footprint: { width: 64, height: 64 },
    path: "M32 0 L64 32 L32 64 L0 32 Z",
    details: [
      "M32 6 L37 10.5 L32 15 L27 10.5 Z",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 1, y: 0.5 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 8, y: 25, width: 48, height: 12 },
    figure: { x: 17, y: 38, width: 30, height: 11 },
    note: "Variante «rombo» de la familia Decisión: el único rombo exacto de la familia, el del catálogo original; la entrada baja por el vértice superior (32,0) y las dos salidas salen por los vértices izquierdo y derecho (0,32) y (64,32), que son las tres anclas de la tabla; el rombo anidado de arriba conserva el «doble rombo» del original y, como la caja inscrita del rombo mide 32 px de ancho (menos de 60), el rótulo va en la banda superior y la cifra en la inferior, en la misma columna. La cifra va en una línea y no apilada: la caja inscrita de 32×32 admite de sobra la zona de una línea de 30 px, y la forma apilada de 14×22 tampoco ayudaría, porque con el subtítulo harían falta 44 px de alto y sólo hay 32. El rombo da 48 px de ancho util a la altura de la banda de rotulo, y la cifra baja a la banda de abajo sin perder su sitio.",
  },
  "Transfer": {
    family: "Decisión",
    variant: "rombo",
    footprint: { width: 64, height: 64 },
    path: "M32 0 L64 32 L32 64 L0 32 Z",
    details: [
      "M28 7 L36 11 L28 15",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 1, y: 0.5 },
      { name: "Salida2", x: 0, y: 0.5 },
    ],
    label: { x: 8, y: 25, width: 48, height: 12 },
    figure: { x: 17, y: 38, width: 30, height: 11 },
    note: "Variante «rombo» de la familia Decisión, la misma silueta de Test: lo que la distingue es la punta de flecha de la corona, que dice que el flujo se reenvía; las tres anclas son las de la tabla (32,0), (64,32) y (0,32), en los vértices, y la caja inscrita de 32 px de ancho obliga al rótulo arriba y la cifra abajo. La cifra va en una línea (zona de 30 px) y no apilada: la apilada de 14×22 cabría de sobra en la caja inscrita, pero no resuelve el problema del subtítulo, que necesita 44 px de alto de los 32 que hay. El rombo da 48 px de ancho util a la altura de la banda de rotulo, y la cifra baja a la banda de abajo sin perder su sitio.",
  },
  "Depart": {
    family: "Cola",
    variant: "cajón con galón",
    footprint: { width: 97, height: 32 },
    path: "M 0 0 H 70 A 27 16 0 0 1 70 32 L 0 32 Z",
    details: [
      "M 70 3 L 70 29",
      "M 76 9 L 85 16",
      "M 76 23 L 85 16",
    ],
    ports: [
      { name: "Entrada1", x: 0.36, y: 0 },
      { name: "Salida1", x: 0.36, y: 1 },
    ],
    label: { x: 5, y: 7, width: 41, height: 12 },
    figure: { x: 47, y: 7, width: 20, height: 12 },
    note: "la misma caja de Queue con galón de salida en el remate, la figura del catálogo original; el galón vive en el remate, así que la cifra se queda en el cuerpo, y entrada y salida comparten el 36% del ancho. El rotulo se ensancha a 41 px para que «Depart» quepa entero en una linea, y la zona de cifra baja a 20 px (para que la cola muestre su contenido, que son hasta tres cifras) sin mover su borde derecho.",
  },
  "Queue": {
    family: "Cola",
    variant: "caja con remate redondeado",
    footprint: { width: 97, height: 32 },
    path: "M 0 0 H 70 A 27 16 0 0 1 70 32 L 0 32 Z",
    details: [
      "M 70 3 L 70 29",
      "M 76 11 L 76 21",
      "M 81 11 L 81 21",
      "M 86 11 L 86 21",
    ],
    ports: [
      { name: "Entrada1", x: 0.36, y: 0 },
      { name: "Salida1", x: 0.36, y: 1 },
    ],
    label: { x: 5, y: 7, width: 31, height: 12 },
    figure: { x: 37, y: 7, width: 30, height: 12 },
    note: "caja con remate redondeado a la derecha, la silueta de la familia Cola; las tres marcas del remate son los que esperan, así que rótulo y cifra se quedan en el cuerpo, y la entrada y la salida comparten el 36% del ancho. El rotulo se ensancha 1 px: «Queue» mide 27 px de monoespaciada y la zona necesita 31 para que quepa entero.",
  },
  "FacilityInt": {
    family: "Recurso",
    variant: "estadio con galón y accesorio",
    footprint: { width: 120, height: 40 },
    path: "M 20,0 H 96 L 120,20 L 96,40 H 20 A 20,20 0 0 1 20,0 Z",
    details: [
      "M 108,13 L 104,20 L 108,27 L 112,20 Z",
    ],
    ports: [
      { name: "Entrada1", x: 0.41, y: 0 },
      { name: "SalidaAlt", x: 1, y: 0.5 },
      { name: "Salida1", x: 0.41, y: 1 },
    ],
    label: { x: 4, y: 10, width: 64, height: 20 },
    figure: { x: 68, y: 10, width: 20, height: 12 },
    note: "El galón cerrado hace de salida lateral: su punta cae en (120,20), el ancla de SalidaAlt de gClasses.xml; el rombo interior conserva el diamante que dibujaba el original. La cifra vive en el cuerpo, a la izquierda del galón, porque en el accesorio está el puerto. El rotulo ocupa 64 px del estadio para que «FacilityInt» quepa entero en una linea, y la zona de cifra baja a 20 px (los de su texto, «82%») sin mover su borde derecho.",
  },
  "FacilityUse": {
    family: "Recurso",
    variant: "estadio con galón",
    footprint: { width: 100, height: 40 },
    path: "M 20,0 H 86 L 100,20 L 86,40 H 20 A 20,20 0 0 1 20,0 Z",
    details: [
      "M 92,14.5 L 98,20 L 92,25.5",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 4, y: 10, width: 65, height: 20 },
    figure: { x: 69, y: 10, width: 20, height: 12 },
    note: "Estadio de extremo redondeado (el RoundRect 0,0,100,40 del editor original) con el galón cerrado a la derecha: la punta dice 'el recurso queda tomado'. Las anclas salen de gClasses.xml (Entrada1 50,0 y Salida1 50,40), sobre las rectas superior e inferior. El rotulo ocupa 65 px del estadio para que «FacilityUse» quepa entero en una linea, y la zona de cifra baja a 20 px (los de su texto, «82%») sin mover su borde derecho.",
  },
  "Preempt": {
    family: "Recurso",
    variant: "galón afilado",
    footprint: { width: 88, height: 32 },
    path: "M 16,0 H 80 L 88,32 H 16 A 16,16 0 0 1 16,0 Z",
    details: [
      "M 76,20 L 82,18",
    ],
    ports: [
      { name: "Entrada1", x: 0.39, y: 0 },
      { name: "Salida1", x: 0.39, y: 1 },
      { name: "SalidaAlt", x: 1, y: 1 },
    ],
    label: { x: 6, y: 9, width: 42, height: 12 },
    figure: { x: 49, y: 10, width: 30, height: 12 },
    note: "El cuerpo se afila hacia el vértice inferior derecho, que es el ancla de SalidaAlt (100%,100%) de gClasses.xml; la marca de corte sobre la diagonal cuenta la interrupción. El corte arranca en x=80 para que la cifra quepa a su izquierda sin pisar el puerto. Bloque de 32 px: sólo rótulo (§2.2).",
  },
  "Release": {
    family: "Recurso",
    variant: "galón abierto",
    footprint: { width: 98, height: 32 },
    path: "M 16,0 H 86 L 98,8.96 L 93.2,16 L 98,23.04 L 86,32 H 16 A 16,16 0 0 1 16,0 Z",
    details: [
      "M 91,11.5 L 87.5,16 L 91,20.5",
    ],
    ports: [
      { name: "Entrada1", x: 0.35, y: 0 },
      { name: "Salida1", x: 0.35, y: 1 },
    ],
    label: { x: 6, y: 9, width: 42, height: 12 },
    figure: { x: 54, y: 10, width: 30, height: 12 },
    note: "Galón abierto: la muesca en V del extremo derecho dice 'el recurso se suelta'. Bloque de 32 px: sólo rótulo, en una línea (§2.2). Las anclas son las de gClasses.xml (35,0 y 35,32). El rotulo se ensancha a 42 px para que «Release» (37,8 px) quepa entero en una linea.",
  },
  "Return": {
    family: "Recurso",
    variant: "galón abierto",
    footprint: { width: 88, height: 32 },
    path: "M 16,0 H 76 L 88,8.96 L 83.2,16 L 88,23.04 L 76,32 H 16 A 16,16 0 0 1 16,0 Z",
    details: [
      "M 81,11.5 L 77.5,16 L 81,20.5",
    ],
    ports: [
      { name: "Entrada1", x: 0.39, y: 0 },
      { name: "Salida1", x: 0.39, y: 1 },
    ],
    label: { x: 5, y: 9, width: 37, height: 12 },
    figure: { x: 44, y: 10, width: 30, height: 12 },
    note: "Galón abierto, igual que Release: soltar y devolver el recurso son la misma operación vista desde los dos lados. Bloque de 32 px: sólo rótulo (§2.2). Las anclas son las de gClasses.xml (35,0 y 35,32). El rotulo se ensancha a 37 px para que «Return» (32,4 px) quepa entero en una linea.",
  },
  "Seize": {
    family: "Recurso",
    variant: "galón cerrado",
    footprint: { width: 98, height: 32 },
    path: "M 16,0 H 86 L 98,16 L 86,32 H 16 A 16,16 0 0 1 16,0 Z",
    details: [
      "M 92,11.5 L 96.5,16 L 92,20.5",
    ],
    ports: [
      { name: "Entrada1", x: 0.35, y: 0 },
      { name: "Salida1", x: 0.35, y: 1 },
    ],
    label: { x: 4, y: 9, width: 31, height: 12 },
    figure: { x: 54, y: 10, width: 30, height: 12 },
    note: "Galón cerrado: la punta derecha dice 'el recurso queda tomado'. Es un bloque de 32 px de alto, asi que lleva sólo el rótulo en una línea (contrato v2 §2.2). Las anclas son las de gClasses.xml (35,0 y 35,32). El rotulo se ensancha a 31 px: «Seize» mide 27 px de monoespaciada y la zona necesita 31 para que quepa entero.",
  },
  "SelectF": {
    family: "Recurso",
    variant: "embudo con galón",
    footprint: { width: 100, height: 50 },
    path: "M 25,0 H 82 L 100,25 L 82,50 H 25 A 25,25 0 0 1 25,0 Z",
    details: [
      "M 6,25 A 5,10 0 1 0 16,25 A 5,10 0 1 0 6,25 Z",
      "M 91,20 L 97,25 L 91,30",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
      { name: "Salida2", x: 1, y: 0.5 },
    ],
    label: { x: 14, y: 13, width: 42, height: 22 },
    figure: { x: 56, y: 13, width: 30, height: 12 },
    note: "La punta del galón cerrado coincide con la salida lateral Salida2 (100%,50%) de gClasses.xml y la elipse de la izquierda conserva la reserva de candidatos del original. SelectF no declara entidad (Param, maximo, minimo, op), asi que su cifra es el par de siempre (contrato v2 §6.1). El rotulo se ensancha a 42 px para que «SelectF» (37,8 px) quepa entero en una linea.",
  },
  "Enter": {
    family: "Depósito",
    variant: "barril",
    footprint: { width: 96, height: 32 },
    path: "M 5.6,0 H 90.4 A 5.6,16 0 0 1 90.4,32 H 5.6 A 5.6,16 0 0 1 5.6,0 Z",
    details: [
      "M 90.4,0 A 3.5,16 0 0 0 90.4,32",
    ],
    ports: [
      { name: "Entrada1", x: 0.36, y: 0 },
      { name: "Salida1", x: 0.36, y: 1 },
    ],
    label: { x: 8, y: 9, width: 32, height: 12 },
    figure: { x: 44, y: 10, width: 32, height: 12 },
    note: "Cilindro tumbado: el original ya dibujaba aquí un extremo cilíndrico, con la boca y las marcas de unidades a la derecha. La cifra va en el centro del barril, lejos de la boca curva (§6.3), y el bloque, de 32 px, lleva sólo el rótulo (§2.2). Anclas 35,0 y 35,32 de gClasses.xml.",
  },
  "Leave": {
    family: "Depósito",
    variant: "barril",
    footprint: { width: 96, height: 32 },
    path: "M 5.6,0 H 90.4 A 5.6,16 0 0 1 90.4,32 H 5.6 A 5.6,16 0 0 1 5.6,0 Z",
    details: [
      "M 90.4,0 A 3.5,16 0 0 0 90.4,32",
    ],
    ports: [
      { name: "Entrada1", x: 0.36, y: 0 },
      { name: "Salida1", x: 0.36, y: 1 },
    ],
    label: { x: 8, y: 9, width: 32, height: 12 },
    figure: { x: 44, y: 10, width: 32, height: 12 },
    note: "Misma silueta que Enter: entrar y egresar del depósito son la misma operación vista desde los dos lados. Cifra en el centro del barril, rótulo sólo (32 px de alto) y anclas 35,0 y 35,32 de gClasses.xml.",
  },
  "SelectS": {
    family: "Depósito",
    variant: "barril con embudo",
    footprint: { width: 100, height: 50 },
    path: "M 8.75,0 H 91.25 A 8.75,25 0 0 1 91.25,50 H 8.75 A 8.75,25 0 0 1 8.75,0 Z",
    details: [
      "M 91.25,0 A 5,25 0 0 0 91.25,50",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
      { name: "Salida2", x: 1, y: 0.5 },
    ],
    label: { x: 10, y: 13, width: 42, height: 22 },
    figure: { x: 53, y: 13, width: 30, height: 12 },
    note: "El punto más a la derecha del barril es (100,25), el ancla de Salida2 (100%,50%) de gClasses.xml; el resto de anclas, 50,0 y 50,50, caen sobre las rectas. SelectS no declara entidad, asi que su cifra es el par de siempre (contrato v2 §6.1). El rotulo se ensancha a 42 px para que «SelectS» quepa entero en una linea, y la zona de cifra conserva su sitio y su ancho.",
  },
  "StorageDef": {
    family: "Depósito",
    variant: "barril en placa (declaración)",
    footprint: { width: 100, height: 30 },
    path: "M 5.25,0 H 94.75 A 5.25,15 0 0 1 94.75,30 H 5.25 A 5.25,15 0 0 1 5.25,0 Z",
    details: [
      "M 14,5 H 86",
      "M 94.75,0 A 3,15 0 0 0 94.75,30",
    ],
    ports: [],
    label: { x: 8, y: 9, width: 60, height: 12 },
    figure: null,
    note: "Placa apagada de la familia Depósito: el barril declara el depósito y no tiene puertos. No lleva cifra: la capacidad es una propiedad del modelo, no del estado de la corrida (contrato v2, corrección 7). Al medir 30 px de alto sigue la regla de §2.2 y muestra sólo el rótulo. El rotulo se ensancha a 60 px para que «StorageDef» (54 px) quepa entero: la declaracion no lleva cifra y el ancho util es todo suyo.",
  },
  "StorageUse": {
    family: "Depósito",
    variant: "barril con nivel",
    footprint: { width: 100, height: 42 },
    path: "M 7,0 H 93 A 7,20 0 0 1 93,40 H 7 A 7,20 0 0 1 7,0 Z",
    details: [
      "M 16,33 H 82",
      "M 93,0 A 4.5,20 0 0 0 93,40",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 0.95 },
    ],
    label: { x: 6, y: 8, width: 44, height: 24 },
    figure: { x: 54, y: 10, width: 32, height: 12 },
    note: "El cilindro se dibuja de 0 a 40 de alto dentro de la huella de 42, porque el ancla de Salida1 (50%,95%) cae en y=40 y debe quedar sobre el borde inferior, como en el editor original (Rectangle 0,0,96,36 + Rectangle 4,4,100,40). «StorageUse» no cabe en una linea del barril, asi que va en dos («Storage» y «Use») y la zona de rotulo sube a 44 px de ancho y 24 px de alto, por encima de la linea de nivel.",
  },
  "QTable": {
    family: "Tabla",
    variant: "hoja con cabecera y cola (declaración)",
    footprint: { width: 130, height: 70 },
    path: "M 5,0 H 125 A 5,5 0 0 1 130,5 V 65 A 5,5 0 0 1 125,70 H 5 A 5,5 0 0 1 0,65 V 5 A 5,5 0 0 1 5,0 Z",
    details: [
      "M 0,20 H 130",
      "M 14,0 V 70",
      "M 116,7 L 122,10 L 116,13",
    ],
    ports: [],
    label: { x: 18, y: 3, width: 46, height: 26 },
    figure: null,
    note: "Placa apagada de la familia Tabla; la flecha de la cabecera recoge el lápiz de cola que el editor original dibujaba a la derecha de QTable. No tiene puertos y, por ser una declaración, no lleva cifra.",
  },
  "Table": {
    family: "Tabla",
    variant: "hoja con cabecera (declaración)",
    footprint: { width: 100, height: 70 },
    path: "M 5,0 H 95 A 5,5 0 0 1 100,5 V 65 A 5,5 0 0 1 95,70 H 5 A 5,5 0 0 1 0,65 V 5 A 5,5 0 0 1 5,0 Z",
    details: [
      "M 0,20 H 100",
      "M 14,0 V 70",
    ],
    ports: [],
    label: { x: 18, y: 3, width: 46, height: 26 },
    figure: null,
    note: "Placa apagada de la familia Tabla: hoja con cabecera y columna de índice, como la dibujaba gClasses.xml (Line 0,16,100,16 y Line 10,0,10,65). El nombre va en la banda de cabecera y el rótulo debajo, como en el original. Sin puertos y sin cifra: es una declaración.",
  },
  "Tabulate": {
    family: "Tabla",
    variant: "hoja con cabecera",
    footprint: { width: 100, height: 50 },
    path: "M 5,0 H 95 A 5,5 0 0 1 100,5 V 45 A 5,5 0 0 1 95,50 H 5 A 5,5 0 0 1 0,45 V 5 A 5,5 0 0 1 5,0 Z",
    details: [
      "M 0,16 H 100",
      "M 14,0 V 50",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 16, y: 18, width: 48, height: 22 },
    figure: { x: 64, y: 18, width: 28, height: 12 },
    note: "El rótulo va en el cuerpo y no en la cabecera porque Entrada1 está en (50,0), en el borde superior de la banda: el texto nunca invade el borde con puerto. La cifra es el número de muestras (§6). Anclas 50,0 y 50,50 de gClasses.xml. El rotulo se ensancha a 48 px para que «Tabulate» (43,2 px) quepa entero, y la zona de cifra baja a 28 px (para el numero de muestras) sin mover su borde derecho.",
  },
  "Link": {
    family: "Cadena",
    variant: "cápsulas enlazadas (cápsula a la derecha)",
    footprint: { width: 98, height: 50 },
    path: "M 15,10 H 60 V 7 A 19,7 0 0 1 98,7 V 43 A 19,7 0 0 1 60,43 V 40 H 15 A 15,15 0 0 1 0,25 A 15,15 0 0 1 15,10 Z",
    details: [
      "M 60,10 V 40",
    ],
    ports: [
      { name: "Entrada1", x: 0.35, y: 0.2 },
      { name: "Salida1", x: 0.35, y: 0.8 },
    ],
    label: { x: 62, y: 11, width: 35, height: 22 },
    figure: { x: 4, y: 17, width: 30, height: 12 },
    note: "Cápsulas enlazadas: la barra horizontal y la cápsula vertical que el original ya superponía (Rectangle 0,10,71,40 + RoundRect 70,0,98,50). El texto va en la cápsula vertical y la cifra en el extremo libre de la barra, porque la barra está partida por los dos puertos. Entrada1 (35,10) y Salida1 (35,40) caen sobre las rectas de la barra; la línea de eslabón marca el solape. Unlink es su calco al espejo.",
  },
  "Unlink": {
    family: "Cadena",
    variant: "cápsulas enlazadas (cápsula a la izquierda)",
    footprint: { width: 98, height: 50 },
    path: "M 83,10 H 38 V 7 A 19,7 0 0 0 0,7 V 43 A 19,7 0 0 0 38,43 V 40 H 83 A 15,15 0 0 0 98,25 A 15,15 0 0 0 83,10 Z",
    details: [
      "M 38,10 V 40",
    ],
    ports: [
      { name: "Entrada1", x: 0.56, y: 0.2 },
      { name: "Salida1", x: 0.14, y: 0.98 },
      { name: "SalidaAlt", x: 0.98, y: 0.52 },
      { name: "Salida2", x: 0.56, y: 0.8 },
    ],
    label: { x: 1, y: 11, width: 36.5, height: 22 },
    figure: { x: 60, y: 17, width: 30, height: 12 },
    note: "Calco al espejo de Link: la misma figura reflejada en horizontal (la comprobación es objetiva: la silueta de Link, reflejada, coincide con esta en el 99,8 % de los píxeles). La barra va a la derecha y la cápsula a la izquierda, y el texto se refleja con ella: el rótulo en la cápsula (1,11) y la cifra en el extremo libre de la barra (60,17). La cifra va 4 px antes del espejo exacto (64) porque su salida alternativa, en el borde derecho, se queda sin sitio si la cifra ocupa la esquina. Sus cuatro anclas —Entrada1 (55,10), Salida2 (55,40), Salida1 (14,49) y SalidaAlt (97,26)— caen cada una en el borde de la silueta. El rotulo se ensancha a 36,5 px, todo lo que da la capsula vertical, para que «Unlink» (32,4 px) quepa entero en una linea.",
  },
  "Alter": {
    family: "Grupo",
    variant: "hexagono con elipse lateral",
    footprint: { width: 100, height: 50 },
    path: "M0 25 L10 0 H70 L80 25 L70 50 H10 Z M80 25 A10 10 0 1 1 100 25 A10 10 0 1 1 80 25 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.4, y: 0 },
      { name: "Salida1", x: 0.1, y: 1 },
      { name: "Salida2", x: 0.7, y: 1 },
    ],
    label: { x: 8, y: 10, width: 34, height: 22 },
    figure: { x: 46, y: 18, width: 30, height: 11 },
    note: "Variante «hexagono con elipse lateral» de la familia Grupo: el hexágono del original con la burbuja del valor nuevo a la derecha; la entrada baja por el borde superior (40,0) y las dos salidas salen por los dos vértices inferiores (10,50) y (70,50); la elipse lateral es la firma de la clase y no lleva texto, así que el rótulo va a la izquierda y la cifra a la derecha dentro del hexágono, y la cifra baja porque a la altura del rótulo el hexágono se estrecha y no da 60 px de ancho.",
  },
  "Assemble": {
    family: "Grupo",
    variant: "embudo",
    footprint: { width: 88, height: 32 },
    path: "M0 4 L4 0 H84 L88 4 L72 32 H16 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 12, y: 13, width: 48, height: 12 },
    figure: { x: 52, y: 1, width: 26, height: 11 },
    note: "Variante «embudo» de la familia Grupo: muchas entradas entran por el borde superior (44,0) y el lote sale por el borde inferior (44,32), y el cuerpo se estrecha hacia abajo como en la cuña del editor original; con 32 px de alto no cabe subtítulo, así que va sólo el rótulo, y el ancho útil (76 px) da para la partición izquierda/derecha. El embudo da 48 px de ancho util en su banda media, y la cifra sube a la banda alta, a la derecha, con 26 px (los de su texto), para no pisar el puerto de salida.",
  },
  "Examine": {
    family: "Grupo",
    variant: "caja sobre cuña",
    footprint: { width: 80, height: 48 },
    path: "M8 0 H72 L78 6 V32 H80 L40 48 L0 32 H2 V6 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.25, y: 0.83 },
      { name: "Salida2", x: 0.75, y: 0.83 },
    ],
    label: { x: 3, y: 8, width: 42, height: 22 },
    figure: { x: 46, y: 10, width: 30, height: 11 },
    note: "Variante «caja sobre cuña»: la misma horquilla de Scan, con la caja más ancha y sin detalle en la cuña; la cuña se dimensiona para que sus dos rectas pasen justo por (20,40) y (60,40), las anclas de salida del catálogo, así que los puertos quedan sobre la arista y no dentro del relleno. El rotulo se ensancha a 42 px para que «Examine» (37,8 px) quepa entero en una linea.",
  },
  "ExamineNumber": {
    family: "Grupo",
    variant: "caja sobre cuña",
    footprint: { width: 80, height: 48 },
    path: "M8 0 H72 L78 6 V32 H80 L40 48 L0 32 H2 V6 Z",
    details: [
      "M36 40 V46 M40 40 V46 M44 40 V46",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.25, y: 0.83 },
      { name: "Salida2", x: 0.75, y: 0.83 },
    ],
    label: { x: 3, y: 8, width: 43, height: 24 },
    figure: { x: 46, y: 10, width: 30, height: 11 },
    note: "Variante «caja sobre cuña», igual que Examine, con las tres rayas de conteo dentro de la cuña (el número que se examina, no la transacción); los anclajes (20,40) y (60,40) siguen sobre las rectas de la cuña, que es lo que exige §3.1. «ExamineNumber» no cabe en una linea de la caja, asi que va en dos («Examine» y «Number») y la zona de rotulo sube a 24 px de alto, por encima de la cuña.",
  },
  "Gather": {
    family: "Grupo",
    variant: "embudo invertido",
    footprint: { width: 88, height: 32 },
    path: "M14 0 H74 L88 28 L84 32 H4 L0 28 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 11, y: 9, width: 38, height: 12 },
    figure: { x: 49, y: 13, width: 30, height: 11 },
    note: "Variante «embudo invertido» de la familia Grupo, el gemelo de Assemble: el grupo se reúne en la base ancha y la entrada (44,0) queda en el borde superior corto; con 32 px de alto sólo cabe el rótulo, y el texto y la cifra se reparten en la banda ancha del cuerpo, por encima de la salida de abajo. El rotulo se ensancha a 38 px para que «Gather» (32,4 px) quepa entero, y la zona de cifra conserva su sitio y su ancho.",
  },
  "JoinMe": {
    family: "Grupo",
    variant: "hexagono con elipse",
    footprint: { width: 100, height: 50 },
    path: "M0 14 H100 V40 L90 50 H10 L0 40 Z M43 7 A7 7 0 1 1 57 7 A7 7 0 1 1 43 7 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0.14 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 4, y: 16, width: 62, height: 21 },
    figure: { x: 68, y: 20, width: 30, height: 11 },
    note: "Variante «hexagono con elipse» de la familia Grupo: cuerpo hexagonal y burbuja de los miembros arriba; la entrada (50,7) cae en el INTERIOR de esa burbuja y no en su borde, que es lo que declara el catálogo original, así que —única excepción a §3.1— la burbuja se dibuja CENTRADA en el ancla para que el puerto quede dentro de la figura y no flotando sobre el relleno; la salida sale por el borde inferior (50,50).",
  },
  "JoinNumber": {
    family: "Grupo",
    variant: "hexagono con elipse",
    footprint: { width: 100, height: 50 },
    path: "M0 14 H100 V40 L90 50 H10 L0 40 Z M43 7 A7 7 0 1 1 57 7 A7 7 0 1 1 43 7 Z",
    details: [
      "M82 33 V39 M88 33 V39 M94 33 V39",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0.14 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 4, y: 16, width: 62, height: 21 },
    figure: { x: 68, y: 20, width: 30, height: 11 },
    note: "Variante «hexagono con elipse»: la misma burbuja de JoinMe, con las tres rayas del contador de «N» en la esquina baja derecha; la entrada sigue en el centro de la burbuja (50,7) y por eso la burbuja se dibuja centrada en el ancla (excepción de §3.1), y la salida en el borde inferior (50,50).",
  },
  "Match": {
    family: "Grupo",
    variant: "cuerpo con mejilla",
    footprint: { width: 80, height: 48 },
    path: "M0 8 L8 0 H72 L80 24 L72 48 H8 L0 40 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 4, y: 8, width: 40, height: 22 },
    figure: { x: 46, y: 18, width: 30, height: 11 },
    note: "Variante «cuerpo con mejilla» de la familia Grupo: el cuerpo hexagonal con la mejilla derecha en punta, que es el triángulo que el original dibujaba sobre el borde derecho para el cotejo de rótulos; la entrada (40,0) y la salida (40,48) caen en los bordes superior e inferior, y el ancho útil (68 px) da para la partición izquierda/derecha.",
  },
  "RemoveMe": {
    family: "Grupo",
    variant: "cuerpo achaflanado con burbuja",
    footprint: { width: 100, height: 50 },
    path: "M0 10 L10 0 H90 L100 10 V36 H0 Z M43 43 A7 7 0 1 1 57 43 A7 7 0 1 1 43 43 Z",
    details: [
      "M8 31 H92",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 0.86 },
      { name: "Salida2", x: 1, y: 0.2 },
    ],
    label: { x: 6, y: 8, width: 58, height: 21 },
    figure: { x: 67, y: 16, width: 30, height: 11 },
    note: "Variante «cuerpo achaflanado con burbuja» de la familia Grupo: cuerpo con las esquinas superiores achaflanadas, la línea de la lista de miembros y la burbuja colgando abajo; la salida Salida1 (50,43) cae en el centro de esa burbuja, igual que la entrada de Join, así que la burbuja se dibuja CENTRADA en el ancla para que el puerto quede dentro de ella (única excepción a §3.1); la entrada sube por (50,0) y la segunda salida sale por el borde derecho (100,10).",
  },
  "RemoveNumber": {
    family: "Grupo",
    variant: "cuerpo achaflanado con burbuja",
    footprint: { width: 100, height: 50 },
    path: "M0 10 L10 0 H90 L100 10 V36 H0 Z M43 43 A7 7 0 1 1 57 43 A7 7 0 1 1 43 43 Z",
    details: [
      "M8 31 H92",
      "M12 33 V36 M18 33 V36 M24 33 V36",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 0.86 },
      { name: "Salida2", x: 1, y: 0.2 },
    ],
    label: { x: 8, y: 6, width: 40, height: 24 },
    figure: { x: 67, y: 16, width: 30, height: 11 },
    note: "Variante «cuerpo achaflanado con burbuja»: igual que RemoveMe, con las tres rayas del contador bajo la línea de la lista, que dicen «N en lugar de uno»; la salida (50,43) sigue en el centro de la burbuja, que por eso se dibuja centrada en el ancla (excepción de §3.1), y las anclas de la entrada (50,0) y de la salida lateral (100,10) son las del catálogo. «RemoveNumber» no cabe en una linea del cuerpo, asi que va en dos («Remove» y «Number»), la zona de rotulo sube a 24 px de alto por encima de la linea de la lista, y la cifra conserva su sitio y su ancho.",
  },
  "RemoveTrans": {
    family: "Grupo",
    variant: "cuerpo achaflanado con burbuja",
    footprint: { width: 100, height: 50 },
    path: "M0 10 L10 0 H90 L100 10 V36 H0 Z M43 43 A7 7 0 1 1 57 43 A7 7 0 1 1 43 43 Z",
    details: [
      "M8 31 H92",
      "M14 35 L26 32",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 0.86 },
      { name: "Salida2", x: 1, y: 0.2 },
    ],
    label: { x: 8, y: 6, width: 40, height: 24 },
    figure: { x: 67, y: 16, width: 30, height: 11 },
    note: "Variante «cuerpo achaflanado con burbuja»: el mismo cuerpo de RemoveMe con una sola raya diagonal bajo la línea, la transacción concreta que sale del grupo; la entrada sube por el borde superior (50,0), la salida lateral por el borde derecho (100,10) y la otra salida queda en el centro de la burbuja, dibujada centrada en el ancla (excepción de §3.1). «RemoveTrans» no cabe en una linea del cuerpo, asi que va en dos («Remove» y «Trans»), la zona de rotulo sube a 24 px de alto por encima de la linea de la lista, y la cifra conserva su sitio y su ancho.",
  },
  "Scan": {
    family: "Grupo",
    variant: "caja sobre cuña",
    footprint: { width: 80, height: 50 },
    path: "M0 6 L6 0 H74 L80 6 V30 L40 50 L0 30 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.25, y: 0.8 },
      { name: "Salida2", x: 0.75, y: 0.8 },
    ],
    label: { x: 6, y: 7, width: 38, height: 20 },
    figure: { x: 48, y: 12, width: 30, height: 11 },
    note: "Variante «caja sobre cuña» de la familia Grupo: la caja con las esquinas superiores achaflanadas sobre la cuña del original, y las dos rectas de la cuña pasan exactamente por las anclas de salida (20,40) y (60,40), de modo que los puntos de puerto caen SOBRE la arista y no flotan dentro del relleno (en un hexágono de familia quedarían dentro); la entrada baja por (40,0).",
  },
  "Select": {
    family: "Selección",
    variant: "embudo",
    footprint: { width: 100, height: 50 },
    path: "M24 0 H72 L100 25 L72 50 H24 V34 A24 9 0 0 1 24 16 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 26, y: 9, width: 54, height: 21 },
    figure: { x: 39, y: 33, width: 30, height: 11 },
    note: "Variante «embudo» de la familia Selección: la boca elíptica izquierda es el depósito de miembros y la punta derecha es lo que sale elegido; la entrada baja por el borde superior (50,0) y la salida sale por el inferior (50,50); el rótulo y la cifra no caben en el mismo lado del embudo, así que van apilados en la misma columna (rótulo arriba, cifra abajo), como manda §2.1.",
  },
  "SelectL": {
    family: "Selección",
    variant: "embudo",
    footprint: { width: 100, height: 50 },
    path: "M24 0 H72 L100 25 L72 50 H24 V34 A24 9 0 0 1 24 16 Z",
    details: [
      "M18 25 A6 6 0 1 1 6 25 A6 6 0 1 1 18 25 Z",
      "M12 25 L16 20",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
      { name: "Salida2", x: 1, y: 0.5 },
    ],
    label: { x: 26, y: 9, width: 54, height: 21 },
    figure: { x: 39, y: 33, width: 30, height: 11 },
    note: "Variante «embudo»: el embudo con el círculo y la palanca del interruptor en la boca; la punta derecha (100,25) es la salida alternativa. Su cifra es el par de siempre (dentro/pasadas) y NO el valor del interruptor: SelectL no declara la entidad que elige (§6.1), así que su cifra por entidad es irresoluble sin una propiedad nueva en el modelo, fuera de este rediseño.",
  },
  "SelectSNA": {
    family: "Selección",
    variant: "embudo",
    footprint: { width: 100, height: 50 },
    path: "M24 0 H72 L100 25 L72 50 H24 V34 A24 9 0 0 1 24 16 Z",
    details: [
      "M18 25 A6 6 0 1 1 6 25 A6 6 0 1 1 18 25 Z",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
      { name: "Salida2", x: 1, y: 0.5 },
    ],
    label: { x: 26, y: 9, width: 54, height: 21 },
    figure: { x: 39, y: 33, width: 30, height: 11 },
    note: "Variante «embudo»: el mismo embudo con un círculo en la boca, el elemento vivo, la clase de entidad que se elige; la punta derecha (100,25) es la segunda salida que declara el catálogo; el rótulo y la cifra van apilados en la misma columna y la cifra es el par de siempre (dentro/pasadas), no un número de entidad.",
  },
  "Advance": {
    family: "Operación",
    variant: "placa base",
    footprint: { width: 80, height: 32 },
    path: "M 7 0 H 73 A 7 7 0 0 1 80 7 V 25 A 7 7 0 0 1 73 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z",
    details: [],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 3, y: 7, width: 44, height: 12 },
    figure: { x: 47, y: 7, width: 30, height: 12 },
    note: "la placa base de la familia Operación, sin trazos interiores: es la operación neutra, la que sólo recibe y pasa; con 32 px de alto no cabe subtítulo, y el puerto entra y sale al 50% del ancho, sobre el borde plano de la placa",
  },
  "Assign": {
    family: "Operación",
    variant: "hoja con pestaña",
    footprint: { width: 100, height: 45 },
    path: "M 16 0 H 84 V 17 H 92 A 8 8 0 0 1 100 25 V 37 A 8 8 0 0 1 92 45 H 8 A 8 8 0 0 1 0 37 V 25 A 8 8 0 0 1 8 17 H 16 Z",
    details: [
      "M 38 8 L 62 8",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 10, y: 18, width: 50, height: 22 },
    figure: { x: 64, y: 22, width: 30, height: 14 },
    note: "hoja con pestaña superior, la silueta del catálogo original; la pestaña lleva la línea del título y el rótulo vive en el cuerpo, con la entrada y la salida al 50% del ancho, sobre la pestaña",
  },
  "Buffer": {
    family: "Operación",
    variant: "placa con ranura",
    footprint: { width: 88, height: 32 },
    path: "M 7 0 H 81 A 7 7 0 0 1 88 7 V 25 A 7 7 0 0 1 81 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z",
    details: [
      "M 4 26 L 84 26",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 3, y: 7, width: 50, height: 12 },
    figure: { x: 55, y: 7, width: 30, height: 12 },
    note: "placa con la ranura del búfer atravesada en el tercio inferior, la línea del catálogo original; con 32 px no cabe subtítulo y el texto se queda arriba de la ranura, y los puertos cruzan el 50% del ancho",
  },
  "Counter": {
    family: "Operación",
    variant: "paralelogramo",
    footprint: { width: 100, height: 40 },
    path: "M 6 0 H 74 L 94 34 A 6 6 0 0 1 88 40 H 26 A 6 6 0 0 1 20 34 L 0 6 A 6 6 0 0 1 6 0 Z",
    details: [
      "M 30 31 L 30 37",
      "M 35 31 L 35 37",
      "M 40 31 L 40 37",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 20, y: 8, width: 44, height: 24 },
    figure: { x: 65, y: 22, width: 20, height: 12 },
    note: "placa inclinada, el paralelogramo del catálogo original: el flujo entra por arriba y sale por abajo, y las tres rayas del pie son el contador; la pendiente manda el rótulo a la derecha y la cifra a la banda baja, y los puertos van al 50% del ancho. El rotulo pasa a 44 px de ancho para que «Counter» quepa en una linea, y la zona de cifra baja a 20 px (para el valor que cuenta) conservando su borde derecho.",
  },
  "Function": {
    family: "Operación",
    variant: "tarjeta de gráfica",
    footprint: { width: 100, height: 100 },
    path: "M 8 0 H 92 A 8 8 0 0 1 100 8 V 92 A 8 8 0 0 1 92 100 H 8 A 8 8 0 0 1 0 92 V 8 A 8 8 0 0 1 8 0 Z",
    details: [
      "M 16 92 L 88 92",
      "M 16 92 L 16 54",
      "M 20 86 L 44 60 L 62 74 L 86 56",
    ],
    ports: [],
    label: { x: 8, y: 6, width: 84, height: 22 },
    figure: { x: 34, y: 32, width: 32, height: 14 },
    note: "tarjeta con ejes y curva, la silueta del catálogo original; la gráfica es la firma de la función y baja a media tarjeta para dejar sitio al rótulo y a la cifra, y sin puertos no hay ancla",
  },
  "FunctionDist": {
    family: "Operación",
    variant: "elipse con abanico",
    footprint: { width: 64, height: 64 },
    path: "M 0 32 A 32 32 0 1 0 64 32 A 32 32 0 1 0 0 32 Z",
    details: [
      "M 28 8 L 32 11 L 36 8",
      "M 30 6.5 L 32 11 L 34 6.5",
      "M 31 5.5 L 32 11 L 33 5.5",
    ],
    ports: [],
    label: { x: 8, y: 20, width: 48, height: 24 },
    figure: { x: 18, y: 46, width: 30, height: 11 },
    note: "elipse con el abanico de la distribución, la silueta del catálogo original; con 64 px de ancho el rótulo va arriba y la cifra abajo, en la misma columna. «FunctionDist» no cabe en una linea de un circulo de 64 px, asi que va en dos («Function» y «Dist»): la zona de rotulo ocupa 48 px de la banda central con 24 px de alto, el abanico sube a la parte alta del circulo para que el texto no lo pise, y la cifra baja al borde inferior con 30 px.",
  },
  "FunctionExpon": {
    family: "Operación",
    variant: "tarjeta de gráfica",
    footprint: { width: 100, height: 100 },
    path: "M 8 0 H 92 A 8 8 0 0 1 100 8 V 92 A 8 8 0 0 1 92 100 H 8 A 8 8 0 0 1 0 92 V 8 A 8 8 0 0 1 8 0 Z",
    details: [
      "M 16 92 L 88 92",
      "M 16 92 L 16 54",
      "M 20 58 C 40 60 58 82 88 90",
    ],
    ports: [],
    label: { x: 8, y: 6, width: 84, height: 22 },
    figure: { x: 34, y: 32, width: 32, height: 14 },
    note: "la misma tarjeta de Function con la curva exponencial en lugar de la quebrada; el rótulo coge los 84 px de la banda alta para que el nombre no se corte, y la gráfica baja a media tarjeta",
  },
  "InitiaSV": {
    family: "Operación",
    variant: "cajón con lengüeta",
    footprint: { width: 100, height: 45 },
    path: "M 8 0 H 92 A 8 8 0 0 1 100 8 V 20 A 8 8 0 0 1 92 28 H 84 V 45 H 16 V 28 H 8 A 8 8 0 0 1 0 20 V 8 A 8 8 0 0 1 8 0 Z",
    details: [
      "M 24 36 L 76 36",
    ],
    ports: [],
    label: { x: 8, y: 3, width: 52, height: 22 },
    figure: { x: 62, y: 10, width: 30, height: 14 },
    note: "el mismo cajón de SaveValue en placa apagada, porque esta clase es la que inicializa ese valor; no tiene puertos y por eso no hay ancla, y la cifra es el valor inicial del modelo",
  },
  "InitialLS": {
    family: "Operación",
    variant: "cuerpo con interruptor",
    footprint: { width: 96, height: 32 },
    path: "M 7 0 H 63 A 7 7 0 0 1 70 7 V 8 H 88 A 8 8 0 0 1 96 16 V 24 H 70 V 25 A 7 7 0 0 1 63 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z",
    details: [
      "M 76 20 L 88 12",
    ],
    ports: [],
    label: { x: 5, y: 4, width: 44, height: 24 },
    figure: { x: 49, y: 7, width: 18, height: 12 },
    note: "el mismo interruptor de Logic en placa apagada, porque fija el valor de partida; sin puertos no hay ancla que repartir, y la cifra es el valor inicial del interruptor. «InitialLS» no cabe en una linea del cuerpo, asi que va en dos («Initial» e «LS») y la zona de rotulo sube a 24 px de alto; la zona de cifra baja a 18 px (los de su texto, «on») sin mover su borde derecho.",
  },
  "Logic": {
    family: "Operación",
    variant: "cuerpo con interruptor",
    footprint: { width: 96, height: 32 },
    path: "M 7 0 H 63 A 7 7 0 0 1 70 7 V 8 H 88 A 8 8 0 0 1 96 16 V 24 H 70 V 25 A 7 7 0 0 1 63 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z",
    details: [
      "M 76 20 L 88 12",
    ],
    ports: [
      { name: "Entrada1", x: 0.39, y: 0 },
      { name: "Salida1", x: 0.39, y: 1 },
    ],
    label: { x: 5, y: 7, width: 32, height: 12 },
    figure: { x: 37, y: 7, width: 30, height: 12 },
    note: "cuerpo con la caja del interruptor a la derecha, la silueta del catálogo original; la palanca dibujada en la caja dice la posición, así que la cifra se queda en el cuerpo, y los puertos pasan al 39% del ancho",
  },
  "MFunction": {
    family: "Operación",
    variant: "tarjeta de gráfica",
    footprint: { width: 100, height: 100 },
    path: "M 8 0 H 92 A 8 8 0 0 1 100 8 V 92 A 8 8 0 0 1 92 100 H 8 A 8 8 0 0 1 0 92 V 8 A 8 8 0 0 1 8 0 Z",
    details: [
      "M 16 92 L 88 92",
      "M 16 92 L 16 54",
      "M 20 86 L 44 60 L 62 74 L 86 56",
      "M 20 72 L 44 44 L 62 58 L 86 44",
    ],
    ports: [],
    label: { x: 8, y: 6, width: 84, height: 22 },
    figure: { x: 34, y: 32, width: 32, height: 14 },
    note: "la tarjeta de Function con dos curvas, que es lo que la distingue de la simple en el catálogo original; declaración en placa apagada, y sin puertos no hay anclas",
  },
  "MFunctionDist": {
    family: "Operación",
    variant: "elipse con aros",
    footprint: { width: 72, height: 72 },
    path: "M 0 36 A 36 36 0 1 0 72 36 A 36 36 0 1 0 0 36 Z",
    details: [
      "M 30.5 45 A 5.5 5.5 0 1 1 41.5 45 A 5.5 5.5 0 1 1 30.5 45 Z",
      "M 33.5 45 A 2.5 2.5 0 1 1 38.5 45 A 2.5 2.5 0 1 1 33.5 45 Z",
    ],
    ports: [],
    label: { x: 9, y: 14, width: 54, height: 24 },
    figure: { x: 24, y: 52, width: 30, height: 11 },
    note: "elipse con aros concéntricos en la banda media, la silueta del catálogo original; en un círculo de 72 px el rótulo va arriba y la cifra abajo. «MFunctionDist» va en dos lineas («MFunction» y «Dist»): la zona de rotulo ocupa 54 px de la banda central con 24 px de alto, los aros bajan al centro de la elipse para que el texto no los pise, y la cifra va al borde inferior con 30 px.",
  },
  "Mark": {
    family: "Operación",
    variant: "tarjeta con lengüeta",
    footprint: { width: 96, height: 36 },
    path: "M 8 0 H 68 A 8 8 0 0 1 76 8 V 9 H 88 A 8 8 0 0 1 96 17 V 21 A 8 8 0 0 1 88 29 H 76 V 28 A 8 8 0 0 1 68 36 H 8 A 8 8 0 0 1 0 28 V 8 A 8 8 0 0 1 8 0 Z",
    details: [
      "M 82 19 A 4 4 0 1 0 90 19 A 4 4 0 1 0 82 19 Z",
    ],
    ports: [
      { name: "Entrada1", x: 0.39, y: 0 },
      { name: "Salida1", x: 0.39, y: 1 },
    ],
    label: { x: 5, y: 7, width: 32, height: 12 },
    figure: { x: 39, y: 7, width: 30, height: 12 },
    note: "tarjeta con lengüeta a la derecha, la silueta del catálogo original; el agujero del remate la firma como marca y la cifra se queda en la tarjeta para no pisarlo, y la entrada y la salida bajan al 39% del ancho",
  },
  "Priority": {
    family: "Operación",
    variant: "cuerpo con cuello",
    footprint: { width: 88, height: 52 },
    path: "M 32 0 H 56 A 6 6 0 0 1 62 6 V 20 H 82 A 6 6 0 0 1 88 26 V 46 A 6 6 0 0 1 82 52 H 6 A 6 6 0 0 1 0 46 V 26 A 6 6 0 0 1 6 20 H 26 V 6 A 6 6 0 0 1 32 0 Z",
    details: [
      "M 34 12 L 34 18",
      "M 42 8 L 42 18",
      "M 50 4 L 50 18",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 4, y: 22, width: 50, height: 22 },
    figure: { x: 54, y: 28, width: 30, height: 14 },
    note: "cuerpo con cuello elevado, la silueta del catálogo original; las tres barras del cuello son los niveles de prioridad y el rótulo vive en el cuerpo, con los puertos al 50% del ancho, sobre el cuello. El cuerpo da 50 px utiles: el rotulo ocupa 44 px («Priority», 43,2 px) y la cifra 30, y los dos caben de lado en el mismo sitio de antes.",
  },
  "SaveValue": {
    family: "Operación",
    variant: "cajón con lengüeta",
    footprint: { width: 100, height: 45 },
    path: "M 8 0 H 92 A 8 8 0 0 1 100 8 V 20 A 8 8 0 0 1 92 28 H 84 V 45 H 16 V 28 H 8 A 8 8 0 0 1 0 20 V 8 A 8 8 0 0 1 8 0 Z",
    details: [
      "M 24 36 L 76 36",
    ],
    ports: [
      { name: "Entrada1", x: 0.5, y: 0 },
      { name: "Salida1", x: 0.5, y: 1 },
    ],
    label: { x: 8, y: 3, width: 54, height: 22 },
    figure: { x: 62, y: 10, width: 30, height: 14 },
    note: "cajón con lengüeta abajo, el espejo de Assign: asignar empuja hacia fuera y guardar deja el valor en el cajón; la cifra es el valor guardado y la lengüeta lleva la ranura, con la entrada y la salida al 50% del ancho. El rotulo ocupa 54 px del cajon para que «SaveValue» quepa entero, y la zona de cifra conserva su sitio y su ancho para el valor que guarda.",
  },
  "Variable": {
    family: "Operación",
    variant: "cinta de datos",
    footprint: { width: 120, height: 32 },
    path: "M 0 0 H 104 L 120 16 L 104 32 H 0 Z",
    details: [
      "M 108 10 L 108 22",
    ],
    ports: [],
    label: { x: 6, y: 10, width: 60, height: 12 },
    figure: { x: 70, y: 10, width: 30, height: 12 },
    note: "cinta de datos con la punta cortada, la silueta del catálogo original; es una declaración sin puertos en placa apagada, y con 32 px de alto el rótulo va en una sola línea",
  },
  "VariableA": {
    family: "Operación",
    variant: "cinta de datos",
    footprint: { width: 120, height: 32 },
    path: "M 0 0 H 104 L 120 16 L 104 32 H 0 Z",
    details: [
      "M 108 10 L 108 22",
    ],
    ports: [],
    label: { x: 6, y: 10, width: 60, height: 12 },
    figure: { x: 70, y: 10, width: 30, height: 12 },
    note: "la misma cinta de Variable: el catálogo original les da la misma silueta y sólo cambia el tipo de dato, así que la diferencia se deja en el rótulo; sin puertos no hay ancla",
  },
  "VariableN": {
    family: "Operación",
    variant: "cinta de datos",
    footprint: { width: 120, height: 40 },
    path: "M 0 0 H 104 L 120 20 L 104 40 H 0 Z",
    details: [
      "M 108 14 L 108 26",
    ],
    ports: [],
    label: { x: 8, y: 6, width: 54, height: 24 },
    figure: { x: 66, y: 10, width: 30, height: 14 },
    note: "la misma cinta de Variable, más alta: la silueta comparte familia y la diferencia de altura dice el tipo; sin puertos no hay ancla, y con 40 px ya caben rótulo y subtítulo. El rotulo se ensancha a 54 px para que «VariableN» quepa entero en una linea, y la cifra conserva su sitio y su ancho.",
  },
  "Note": {
    family: "Anotación",
    variant: "hoja con esquina doblada",
    footprint: { width: 80, height: 80 },
    path: "M 0 0 H 58 L 80 22 V 80 H 0 Z",
    details: [
      "M 58 0 V 22 H 80",
    ],
    ports: [],
    label: { x: 10, y: 10, width: 52, height: 26 },
    figure: null,
    note: "hoja con la esquina doblada, la silueta del catálogo original; cambia de familia: pasa de Operación a la familia propia «Anotación» (gris #64748b) porque una anotación no es un paso del flujo, y no tiene puertos ni cifra",
  },
};

/** The classes the module knows, grouped by family. */
export const SHAPE_CLASSES: string[] = [
  "Clock",
  "Generate",
  "Terminate",
  "GateF",
  "GateL",
  "GateM",
  "GateS",
  "Loop",
  "Multifurcacion",
  "MultifurcacionP",
  "Split",
  "Test",
  "Transfer",
  "Depart",
  "Queue",
  "FacilityInt",
  "FacilityUse",
  "Preempt",
  "Release",
  "Return",
  "Seize",
  "SelectF",
  "Enter",
  "Leave",
  "SelectS",
  "StorageDef",
  "StorageUse",
  "QTable",
  "Table",
  "Tabulate",
  "Link",
  "Unlink",
  "Alter",
  "Assemble",
  "Examine",
  "ExamineNumber",
  "Gather",
  "JoinMe",
  "JoinNumber",
  "Match",
  "RemoveMe",
  "RemoveNumber",
  "RemoveTrans",
  "Scan",
  "Select",
  "SelectL",
  "SelectSNA",
  "Advance",
  "Assign",
  "Buffer",
  "Counter",
  "Function",
  "FunctionDist",
  "FunctionExpon",
  "InitiaSV",
  "InitialLS",
  "Logic",
  "MFunction",
  "MFunctionDist",
  "Mark",
  "Priority",
  "SaveValue",
  "Variable",
  "VariableA",
  "VariableN",
  "Note",
];

/** The shape of a class, or `undefined` when the class has no shape yet. */
export function shapeOf(className: string): BlockShape | undefined {
  return BLOCK_SHAPES[className];
}

/** The three colour levels of a family. */
export function familyColors(family: Family): FamilyColors {
  return FAMILY_COLORS[family];
}

/** The three colour levels of the family of a class. */
export function colorsOf(className: string): FamilyColors | undefined {
  const shape = BLOCK_SHAPES[className];
  return shape === undefined ? undefined : FAMILY_COLORS[shape.family];
}
