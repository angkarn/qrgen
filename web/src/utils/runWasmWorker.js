import {
  File,
  Directory,
  OpenFile,
  ConsoleStdout,
  PreopenDirectory,
  WASI,
  strace,
} from "@bjorn3/browser_wasi_shim/dist/index.js";

// class XtermStdio extends Fd {
//   /*:: term: Terminal*/

//   constructor(term /*: Terminal*/) {
//     super();
//     this.term = term;
//   }
//   fd_write(data /*: Uint8Array*/) /*: {ret: number, nwritten: number}*/ {
//     let nwritten = 0;
//     this.term.write(data);
//     return { ret: 0, nwritten: data.byteLength };
//   }
// }

// Convert Map items to File or Directory
function convertMapDirStruct(mapInput) {
  let outputMap = new Map();
  mapInput.forEach((value, key, map) => {
    if (value.contents) {
      outputMap.set(key, new Directory(convertMapDirStruct(value.contents)));
    } else {
      outputMap.set(key, new File(new Int8Array(value.data)));
    }
  });
  return outputMap;
}

async function runWasm({ wasm, importRootFiles, args }) {
  const mapDirStruct = convertMapDirStruct(importRootFiles);

  let env = ["FOO=bar"];
  let fds = [
    new OpenFile(new File([])),
    ConsoleStdout.lineBuffered((msg) => {
      postMessage({ type: "stdout", message: msg });
    }),
    ConsoleStdout.lineBuffered((msg) => {
      postMessage({ type: "stderr", message: msg });
    }),
    new PreopenDirectory(".", mapDirStruct),
  ];

  const rootDir = fds[3].dir.contents;

  const w = new WASI(args, env, fds, { debug: false });

  let next_thread_id = 1;

  const module = new WebAssembly.Module(wasm);

  let inst = await WebAssembly.instantiate(module, {
    env: {
      memory: new WebAssembly.Memory({
        initial: 256,
        maximum: 16384,
        shared: true,
      }),
    },
    wasi: {
      "thread-spawn": function (start_arg) {
        let thread_id = next_thread_id++;
        inst.exports.wasi_thread_start(thread_id, start_arg);
        return thread_id;
      },
    },
    wasi_snapshot_preview1: strace(w.wasiImport, ["fd_prestat_get"]),
  });

  try {
    w.start(inst);
  } catch (e) {
    postMessage({ type: "wasmError", message: "Exception: " + e.message });
  }
  postMessage({ type: "wasmLoaded", rootDir });
}

onmessage = (e) => {
  console.log("runWasmWorker onmessage:", e.data);
  runWasm(e.data);
};
