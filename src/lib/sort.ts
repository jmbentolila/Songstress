export function sortKey(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/^the\s+/, "")
    .normalize("NFD")
    .replace(/\p{M}/gu, "");
}

export function compareByName(a: string, b: string): number {
  return sortKey(a).localeCompare(sortKey(b));
}
