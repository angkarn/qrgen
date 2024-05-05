import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { FitAddon } from "@xterm/addon-fit";
import fileType from "./utils/fileType";
import readFileToUint8Array from "./utils/readFileToUint8Array";
import dirDownload from "./utils/dirDownload";

const term = new Terminal({
  convertEol: true,
  cursorBlink: true,
});

window.term = term;

term.open(document.getElementById("terminal"));

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);
fitAddon.fit();
addEventListener("resize", () => fitAddon.fit());

const appName = "qrgen";
let wasm = null;
let importRootFiles = new Map();

const $ = (q) => document.querySelector(q);

function ael(el, event, fn, options = false) {
  el && el.addEventListener(event, (e) => fn(e), options);
}

let guiInputMode = "gen";

function generateGuiArg() {
  let args = ["qrgen"];

  if (guiInputMode == "gen") {
    args.push(
      "gen",
      `"${$("#contentInput").value}"`,
      "-f",
      "png",
      "-o",
      "gui_output"
    );
    if ($("#topTextInput").value) {
      args.push("-t", `"${$("#topTextInput").value}"`);
    }
    if ($("#bottomTextInput").value) {
      args.push("-b", `"${$("#bottomTextInput").value}"`);
    }
    if ($("#fontSizeInput").value) {
      args.push("--fs", `"${$("#fontSizeInput").value}"`);
    }
    if ($("#guiFontFile").value) {
      args.push("--fp", "gui_font.ttf");
    }
    if ($("#imageSizeInput").value) {
      args.push("-s", $("#imageSizeInput").value);
    }
    return args;
  }

  if (guiInputMode == "from") {
    args.push("from", `gui_content.csv`, "-f", "png", "-o", "gui_output");

    if ($("#contentTemplateInput").value) {
      args.push("--tc", `"${$("#contentTemplateInput").value}"`);
    }
    if ($("#topTextTemplateInput").value) {
      args.push("--ttt", `"${$("#topTextTemplateInput").value}"`);
    }
    if ($("#bottomTextTemplateInput").value) {
      args.push("--ttb", `"${$("#bottomTextTemplateInput").value}"`);
    }
    if ($("#fileNameTemplateInput").value) {
      args.push("--tfn", `"${$("#fileNameTemplateInput").value}"`);
    }
    if ($("#guiFontFile").value) {
      args.push("--fp", "gui_font.ttf");
    }
    if ($("#fontSizeInput").value) {
      args.push("--fs", `"${$("#fontSizeInput").value}"`);
    }
    if ($("#imageSizeInput").value) {
      args.push("-s", $("#imageSizeInput").value);
    }

    return args;
  }
}

const guiSection = $("#guiSection");

$("#closeBtn").addEventListener("click", () => {
  $("#action").style.display = "none";
});

$("#showActionBtn").addEventListener("click", () => {
  $("#action").style.display = "block";
});

$("#closePreviewBtn").addEventListener("click", () => {
  $("#preview").style.display = "none";
});

ael($("#guiCloseBtn"), "click", () => (guiSection.style.display = "none"));

ael($("#guiBtn"), "click", () => (guiSection.style.display = "block"));

ael($("#guiGenerateBtn"), "click", () => {
  commandGenerated.innerHTML = generateGuiArg().join(" ");
});

ael($("#guiGenerateRunBtn"), "click", () => {
  commandGenerated.innerHTML = generateGuiArg().join(" ");
  term.write(generateGuiArg().join(" "));
  runApp(fixQuoteArgs(generateGuiArg()));
});

ael($("#genInput"), "change", (e) => {
  guiInputMode = "gen";
  $("#forFrom").style.display = "none";
  $("#forGen").style.display = "block";
});

ael($("#fromInput"), "change", (e) => {
  guiInputMode = "from";
  $("#forGen").style.display = "none";
  $("#forFrom").style.display = "block";
});

ael($("#uploadBtn"), "click", () => $("#file").click());

ael($("#guiUploadContentBtn"), "click", () => $("#guiContentFile").click());
ael($("#guiUploadFontBtn"), "click", () => $("#guiFontFile").click());
ael(
  $("#guiContentFile"),
  "change",
  async (e) => {
    const readResult = await readFileToUint8Array(e.target.files[0]);
    importRootFiles.set("gui_content.csv", { data: readResult[1] });
    reloadDir(importRootFiles);
  },
  false
);

// find exe from file name
// /(?:\.([^.]+))?$/.exec(FILE_NAME)[1]

ael(
  $("#guiFontFile"),
  "change",
  async (e) => {
    const readResult = await readFileToUint8Array(e.target.files[0]);
    importRootFiles.set("gui_font.ttf", { data: readResult[1] });
    reloadDir(importRootFiles);
  },
  false
);

ael(
  $("#file"),
  "change",
  async (e) => {
    await Promise.all(
      Array.from(e.target.files).map(async (file) => {
        const readResult = await readFileToUint8Array(file);
        importRootFiles.set(readResult[0], { data: readResult[1] });
      })
    );
    reloadDir(importRootFiles);
  },
  false
);

function reloadDir(rootDir) {
  $("#dir").innerHTML = "";
  $("#dir").innerHTML += loopDir(rootDir);
}

function loopDir(dir, queryDir = "") {
  let renderDir = "";
  dir.entries().forEach((e) => {
    if (!!e[1].contents) {
      let query = queryDir ? queryDir + "/" + e[0] : e[0];
      renderDir += `<div class="wrapDir"><div class="item dir"><div>${
        e[0]
      }/</div><a class="dirDownload" data-query-dir="${query}" target='_blank' download='${
        e[0]
      }'>💾</a></div><div class="subdir">${loopDir(
        e[1].contents,
        query
      )}</div></div>`;
    } else {
      const urlData = URL.createObjectURL(new Blob([e[1].data]));
      const type = fileType(e[1].data);
      renderDir += `<div class="item file"><div data-file-type="${type}" data-file="${urlData}">${e[0]}</div><a target='_blank' download='${e[0]}' href='${urlData}'>💾</a></div>`;
    }
  });
  return renderDir;
}

function loading() {
  isStdError = false;
  console.log("loading");
  $("#processing").style.display = "block";
  $("#guiDownloadBtn").style.display = "none";
  $("#guiErrorResult").style.display = "none";
  try {
    importRootFiles.delete("gui_output");
  } catch {}
}

ael($("#guiDownloadBtn"), "click", () => {
  dirDownload(importRootFiles, "gui_output", "qrgen_gui_output");
});

function loaded() {
  $("#processing").style.display = "none";
  console.log("loaded");

  if (isStdError) {
    $("#guiErrorResult").style.display = "block";
  }

  if (importRootFiles?.get("gui_output")?.contents) {
    $("#guiDownloadBtn").style.display = "inline-block";
  }

  term.write("\r\n> ");

  reloadDir(importRootFiles);

  document.querySelectorAll(".dirDownload").forEach((e) => {
    e.addEventListener("click", () => {
      dirDownload(
        importRootFiles,
        e.dataset.queryDir,
        e.dataset.queryDir.replaceAll("/", "_")
      );
    });
  });
}

let isStdError = false;

async function runApp(args = []) {
  if (args[0] !== "qrgen") {
    term.write("> ");
    return;
  }
  term.writeln("");
  console.log("runApp", { args });
  loading();

  const runWorker = new Worker(
    new URL("utils/runWasmWorker.js", import.meta.url),
    { type: "module" }
  );

  runWorker.onmessage = (e) => {
    switch (e.data.type) {
      case "wasmLoaded":
        importRootFiles = e.data.rootDir;
        loaded();
        break;
      case "wasmError":
        isStdError = true;
        term.writeln(e.data.message);
        break;
      case "stdout":
        term.writeln(e.data.message);
        break;
      case "stderr":
        isStdError = true;
        term.writeln(e.data.message);
        break;
      default:
        console.log("out case:", e.data);
    }
  };

  runWorker.postMessage({ wasm, importRootFiles, args });
}

const imgPreview = $("#imgPreview");
const iframePreview = $("#iframePreview");

document.addEventListener("click", (e) => {
  const targetDataFileElm = e.target.closest("[data-file]"); // Or any other selector.
  if (targetDataFileElm) {
    imgPreview.style.display = "none";
    iframePreview.style.display = "none";
    imgPreview.src = "";
    iframePreview.src = "";
    let src = targetDataFileElm.dataset.file;
    if (targetDataFileElm.dataset.fileType == "unknown") {
      iframePreview.src = src;
      iframePreview.style.display = "block";
    } else {
      imgPreview.src = src;
      imgPreview.style.display = "block";
    }
    document.getElementById("preview").style.display = "flex";
    return;
  }
});

function fixQuoteArgs(args) {
  // fix quote arg
  return args.map((e) => {
    const fl = e.replace(/^(.).*(.)$/, "$1$2");
    if (fl == `""` || fl == `''`) return e.slice(1, -1);
    return e;
  });
}

(async function () {
  term.writeln(`\x1b[95mQR Generate by angkarn\x1B[0m`);
  term.writeln(`\x1b[96m Demo of build target to WASI.\x1B[0m\n`);

  term.writeln(`\x1B[93mDownloading\x1B[0m`);
  wasm = await (await fetch("qrgen.wasm")).arrayBuffer();
  term.writeln(`\x1B[92mAlready\x1B[0m\n`);

  let lineBuffer = [];
  let history = [];
  let shellListener = null;
  let offset = 0;

  async function simpleShell(data) {
    // string splitting is needed to also handle multichar input (eg. from copy)
    for (let i = 0; i < data.length; ++i) {
      const c = data[i];
      if (c === "\r") {
        // <Enter> was pressed case
        offset = 0;
        term.write("\r\n");
        if (lineBuffer.length) {
          // we have something in line buffer, normally a shell does its REPL logic here
          // for simplicity - just join characters and exec...
          const command = lineBuffer.join("");
          lineBuffer.length = 0;
          history.push(command);
          try {
            // tricky part: for interactive sub commands you have to detach the shell listener
            // temporarily, and re-attach after the command was finished
            shellListener?.dispose();

            // process string to args
            let args = command.match(
              /("[^"\\]*(?:\\[\S\s][^"\\]*)*"|'[^'\\]*(?:\\[\S\s][^'\\]*)*'|\/[^\/\\]*(?:\\[\S\s][^\/\\]*)*\/[gimy]*(?=\s|$)|(?:\\\s|\S)+)/g
            );

            // fix quote arg
            args = fixQuoteArgs(args);

            runApp(args);
            //await exec(command);  // issue: cannot force-kill in JS (needs to be a good citizen)
          } catch (e) {
            // we have no real process separation with STDERR
            // simply catch any error and output in red
            const msg = !e ? "Unknown Error..." : e.message || e;
            term.write(`\x1b[31m${msg.replace("\n", "\r\n")}\x1b[m`);
          } finally {
            // in any case re-attach shell
            shellListener = term.onData(simpleShell);
          }
        } else {
          term.write("> ");
        }
      } else if (c === "\x7F") {
        // <Backspace> was pressed case
        if (lineBuffer.length) {
          if (offset === 0) {
            lineBuffer.pop();
            term.write("\b \b");
          } else if (offset < 0 && Math.abs(offset) !== lineBuffer.length) {
            let insert = "";

            for (
              let ci = lineBuffer.length + offset;
              ci < lineBuffer.length;
              ci++
            ) {
              insert += lineBuffer[ci];
            }

            lineBuffer.splice(lineBuffer.length + offset - 1, 1);

            let lefts = "";

            for (var ci = 0; ci < insert.length; ci++) {
              lefts += "\x1b[1D";
            }

            const termInsert = "\b \b" + insert + " " + "\b \b" + lefts;
            term.write(termInsert);
          }
        }
      } else if (
        ["\x1b[A", "\x1b[B", "\x1b[C", "\x1b[D"].includes(data.slice(i, i + 3))
      ) {
        // <arrow> keys pressed
        if (data.slice(i, i + 3) === "\x1b[A") {
          // UP pressed, select backwards from history + erase terminal line + write history entry
        } else if (data.slice(i, i + 3) === "\x1b[B") {
          // DOWN pressed, select forward from history + erase terminal line + write history entry
        } else if (data.slice(i, i + 3) === "\x1b[C") {
          if (offset < 0) {
            term.write("\x1b[1C");
            offset++;
          }
        } else if (data.slice(i, i + 3) === "\x1b[D") {
          if (Math.abs(offset) < lineBuffer.length) {
            term.write("\x1b[1D");
            offset--;
          }
        }

        i += 2;
      } else {
        // push everything else into the line buffer and echo back to user

        let insert = "";
        insert += c;

        for (
          let ci = lineBuffer.length + offset;
          ci < lineBuffer.length;
          ci++
        ) {
          insert += lineBuffer[ci];
        }

        let shift = "";

        if (offset < 0) {
          for (
            let ci = lineBuffer.length + offset;
            ci < lineBuffer.length;
            ci++
          ) {
            shift += "\x1b[1D";
          }
        }

        if (offset === 0) {
          lineBuffer.push(c);
        } else if (offset < 0) {
          lineBuffer.splice(lineBuffer.length + offset, 0, c);
        }

        let termInsert = insert;

        if (offset < 0) {
          termInsert += shift;
        }

        term.write(termInsert);
      }
    }
  }

  shellListener = term.onData(simpleShell);

  // start with help
  term.writeln("> qrgen help");
  runApp(["qrgen", "help"]);
})();
