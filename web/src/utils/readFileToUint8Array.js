export default function readFileToUint8Array(file) {
  return new Promise((resolve) => {
    const reader = new FileReader();
    reader.onloadend = async (re) => {
      const fileInt8Array = new Int8Array(re.target.result);
      resolve([file.name, fileInt8Array]);
    };
    reader.readAsArrayBuffer(file);
  });
}
