// Smoke test for the browser package: node crates/qrgen-wasm/smoke.mjs <package dir>
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const dir = resolve(process.argv[2] ?? "crates/qrgen-wasm/pkg");
const { default: init, Batch, version } = await import(pathToFileURL(`${dir}/qrgen.js`));
await init({ module_or_path: readFileSync(`${dir}/qrgen_bg.wasm`) });

const check = (ok, message) => {
  if (!ok) throw new Error(message);
};
for (const [format, name, mimeType, magic] of [
  ["png", "qr.png", "image/png", [0x89, 0x50, 0x4e, 0x47]],
  ["jpeg", "qr.jpg", "image/jpeg", [0xff, 0xd8, 0xff]],
]) {
  const image = new Batch(`{ data: "hello", output: { format: "${format}" } }`, new Map()).render(0);
  check(image.name === name, `${format}: name ${image.name}`);
  check(image.mimeType === mimeType, `${format}: mimeType ${image.mimeType}`);
  check(image.data.length > 100 && magic.every((b, i) => image.data[i] === b), `${format}: bad data`);
}
check(existsSync(`${dir}/OFL.txt`), "OFL.txt (license of the built-in font) is missing");
const rows = new Batch('{ csv: "a.csv", data: "{{1}}" }', new Map([["a.csv", new TextEncoder().encode("x\ny\n")]]));
check(rows.length === 2 && rows.name(1) === "2.png", "csv rows");
console.log(`qrgen ${version()}: ok`);
