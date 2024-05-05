import JSZip from "jszip";

export default function dirDownload(dir, query = "", filename = "output") {
  const zip = new JSZip();
  let queryDir = dir;
  query.split("/").forEach((e) => {
    queryDir = queryDir.get(e).contents;
  });

  deepDirZip(zip, queryDir);

  zip
    .generateAsync({ type: "blob" })
    .then((blob) => {
      const link = document.createElement("a");
      link.style.display = "none";
      document.body.appendChild(link);
      const objectURL = URL.createObjectURL(blob);
      link.href = objectURL;
      link.href = URL.createObjectURL(blob);
      link.download = filename;
      link.click();
    })
    .catch((e) => console.log(e));

  function deepDirZip(zip, dir) {
    const _zip = zip;
    dir.entries().forEach((e) => {
      if (!!e[1].contents) {
        const zipDir = _zip.folder(e[0]);
        deepDirZip(zipDir, e[1].contents);
      } else {
        _zip.file(e[0], new Blob([e[1].data]));
      }
    });
    return _zip;
  }
}
