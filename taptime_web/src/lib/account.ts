import { Weekday } from "@taptime/proto/taptime/weekday_pb.js";

export const DELETE_TIME_DATA_CONFIRMATION = "DELETE DATA";
export const DELETE_PROFILE_CONFIRMATION = "DELETE PROFILE";

export const WEEKDAY_LABELS: Record<Weekday, string> = {
  [Weekday.MONDAY]: "Mon",
  [Weekday.TUESDAY]: "Tue",
  [Weekday.WEDNESDAY]: "Wed",
  [Weekday.THURSDAY]: "Thu",
  [Weekday.FRIDAY]: "Fri",
  [Weekday.SATURDAY]: "Sat",
  [Weekday.SUNDAY]: "Sun",
  [Weekday.UNSPECIFIED]: "",
};

type TimestampLike = {
  seconds: bigint;
  nanos: number;
};

export type ParsedUid = {
  bytes: Uint8Array;
  label: string;
};

export function formatUid(bytes?: Uint8Array) {
  if (!bytes || bytes.length === 0) return "";
  return [...bytes]
    .map((byte) => byte.toString(16).padStart(2, "0").toUpperCase())
    .join(" ");
}

export function parseUid(value: string): ParsedUid | null {
  const cleaned = value.replace(/[\s:;.,-]/g, "").toUpperCase();
  if (cleaned.length === 0) return null;
  if (![8, 14, 20].includes(cleaned.length)) return null;
  if (!/^[0-9A-F]+$/.test(cleaned)) return null;

  const bytes = new Uint8Array(cleaned.length / 2);
  for (let i = 0; i < bytes.length; i += 1) {
    bytes[i] = Number.parseInt(cleaned.slice(i * 2, i * 2 + 2), 16);
  }
  return { bytes, label: formatUid(bytes) };
}

export function timestampLabel(value?: TimestampLike) {
  if (!value) return "Never";
  const ms = Number(value.seconds) * 1000 + Math.floor(value.nanos / 1_000_000);
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(ms));
}

export function weekdayKey(days: Weekday[] | undefined) {
  return [...(days ?? [])].sort((a, b) => a - b).join(",");
}

export function weekdayLabel(days: Weekday[]) {
  return days.length > 0
    ? [...days]
        .sort((a, b) => a - b)
        .map((day) => WEEKDAY_LABELS[day])
        .filter(Boolean)
        .join(", ")
    : "None";
}
