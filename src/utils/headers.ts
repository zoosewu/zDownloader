// The request headers a task carries, as the three fields people actually set.
export type HeaderPairs = Array<[string, string]>;

export interface HeaderFields {
  userAgent: string;
  referer: string;
  cookie: string;
}

const FIELDS: Record<keyof HeaderFields, string> = {
  userAgent: 'User-Agent',
  referer: 'Referer',
  cookie: 'Cookie',
};

export const emptyHeaderFields: HeaderFields = { userAgent: '', referer: '', cookie: '' };

/** Split `headers` into the editable fields and the rest (kept as they are). */
export function toHeaderFields(headers: HeaderPairs): { fields: HeaderFields; rest: HeaderPairs } {
  const fields = { ...emptyHeaderFields };
  const rest: HeaderPairs = [];
  for (const [name, value] of headers) {
    const field = (Object.keys(FIELDS) as Array<keyof HeaderFields>).find(
      (key) => FIELDS[key].toLowerCase() === name.toLowerCase()
    );
    if (field) fields[field] = value;
    else rest.push([name, value]);
  }
  return { fields, rest };
}

/** The headers for `fields` (empty ones left out) followed by `rest`. */
export function fromHeaderFields(fields: HeaderFields, rest: HeaderPairs = []): HeaderPairs {
  const set = (Object.keys(FIELDS) as Array<keyof HeaderFields>)
    .map((key): [string, string] => [FIELDS[key], fields[key].trim()])
    .filter(([, value]) => value !== '');
  return [...set, ...rest];
}
