export default function fileType(file) {
    const arr = file.subarray(0, 4);
    let header = "";
    for (let i = 0; i < arr.length; i++) {
      header += arr[i].toString(16);
    }
  
    switch (header) {
      case "89504e47":
        return "image/png";
      case "47494638":
        return "image/gif";
      case "ffd8ffe0":
      case "ffd8ffe1":
      case "ffd8ffe2":
      case "ffd8ffe3":
      case "ffd8ffe8":
        return "image/jpeg";
      default:
        return "unknown"; // Or you can use the blob.type as fallback
    }
  }