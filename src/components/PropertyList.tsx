// Two-column property sheet (label | value), as in a desktop app's properties pane.
import React from 'react';
import { Box } from '@mui/material';

export interface Property {
  label: string;
  value: React.ReactNode;
  /** Paths, ids: monospace. */
  mono?: boolean;
}

/** Falsy entries (`cond && {...}`) are skipped. */
const PropertyList: React.FC<{ items: Array<Property | false | '' | null | undefined> }> = ({ items }) => (
  <Box
    component="dl"
    sx={{
      m: 0,
      display: 'grid',
      gridTemplateColumns: 'max-content 1fr',
      columnGap: 2,
      rowGap: 0.75,
      fontSize: '0.8125rem',
      '& dt': { color: 'text.secondary', whiteSpace: 'nowrap' },
      '& dd': { m: 0, minWidth: 0, wordBreak: 'break-all', userSelect: 'text' },
    }}
  >
    {items.filter((p): p is Property => !!p).map((p) => (
      <React.Fragment key={p.label}>
        <dt>{p.label}</dt>
        <Box component="dd" sx={p.mono ? { fontFamily: 'Consolas, monospace', fontSize: '0.75rem', lineHeight: 1.7 } : undefined}>
          {p.value}
        </Box>
      </React.Fragment>
    ))}
  </Box>
);

export default PropertyList;
