// 极简 nanoid
export function nanoid(size = 12): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let s = "";
  const arr = new Uint8Array(size);
  crypto.getRandomValues(arr);
  for (let i = 0; i < size; i++) {
    s += chars[arr[i] % chars.length];
  }
  return s;
}
