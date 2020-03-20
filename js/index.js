

const cayley_wasm = import("../pkg/index.js");
console.log('cayley_wasm', cayley_wasm);

(async () => {
    let gremlin_db = await cayley_wasm;

    console.log('gremlin_db', gremlin_db);

    let db = gremlin_db.NewMemoryGraph();

    let g = db.graph();

    console.log('g', g);

    let path = g.V();

    console.log('path', path);


    console.log('db.read()', db.read());

})();
