export function formatIsk(value: number): string {
  if (value >= 1e12) return (value / 1e12).toFixed(1) + "T";
  if (value >= 1e9) return (value / 1e9).toFixed(1) + "B";
  if (value >= 1e6) return (value / 1e6).toFixed(1) + "M";
  if (value >= 1e3) return (value / 1e3).toFixed(0) + "K";
  return value.toFixed(0);
}

export function getMonthName(month: number): string {
  const months = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ];
  return months[month - 1] || "???";
}

export function getThreatClass(level: string): string {
  return `threat-${level.toLowerCase()}`;
}

export function getKdRatio(destroyed: number, lost: number): string {
  if (lost === 0) return destroyed > 0 ? "∞" : "0";
  return (destroyed / lost).toFixed(1);
}

export function getPpk(points: number, kills: number): number {
  if (kills === 0) return 0;
  return points / kills;
}

export function formatPpk(points: number, kills: number): string {
  const ppk = getPpk(points, kills);
  if (ppk >= 1000) return (ppk / 1000).toFixed(1) + "k";
  return ppk.toFixed(0);
}

export function openZkill(characterId: number): void {
  window.open(`https://zkillboard.com/character/${characterId}/`, "_blank");
}

export function getPortraitUrl(characterId: number, size = 32): string {
  return `https://images.evetech.net/characters/${characterId}/portrait?size=${size}`;
}

export function getShipIconUrl(typeId: number, size = 32): string {
  return `https://images.evetech.net/types/${typeId}/icon?size=${size}`;
}


