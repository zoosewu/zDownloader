// Material UI theme: a dense desktop-application look (small radii, compact controls, no
// ripples or shadows), in light and dark.
//
// Every color comes from one brand hue, the app icon's teal-emerald. Status colors keep the
// brand's saturation and lightness and only turn the hue, so they read as one family; greys
// carry a trace of the brand hue. Components use palette names (primary, info, success,
// warning, error, text.*, divider), never color literals.
import { createTheme, alpha, type PaletteMode, type Theme } from '@mui/material/styles';

/** The icon's teal-emerald (#0F766E to #34D399). */
export const BRAND_HUE = 165;

// Comma syntax: MUI's color helpers (alpha, darken, getContrastText) do not parse `hsl(h s l)`.
const hsl = (h: number, s: number, l: number) => `hsl(${h}, ${s}%, ${l}%)`;

/** Hue offsets from the brand: downloading (info) leans blue, warning amber, error rose. */
function tones(dark: boolean) {
  // Same saturation and lightness per mode, so no color shouts louder than the brand.
  const [s, l] = dark ? [62, 58] : [72, 33];
  const tone = (offset: number) => hsl((BRAND_HUE + offset + 360) % 360, s, l);
  return {
    primary: tone(0),
    info: tone(40),
    warning: tone(-125),
    error: tone(-170),
  };
}

function appTheme(mode: PaletteMode): Theme {
  const dark = mode === 'dark';
  const t = tones(dark);
  const grey = (s: number, l: number) => hsl(BRAND_HUE, s, l);
  const base = createTheme({
    spacing: 6,
    shape: { borderRadius: 4 },
    palette: {
      mode,
      primary: { main: t.primary },
      secondary: { main: t.info },
      // Finished is the brand's own color; running is told apart by `info`.
      success: { main: t.primary },
      info: { main: t.info },
      warning: { main: t.warning },
      error: { main: t.error },
      background: dark ? { default: grey(9, 9), paper: grey(8, 12) } : { default: grey(14, 95), paper: grey(20, 99.5) },
      text: dark
        ? { primary: grey(12, 91), secondary: grey(7, 63), disabled: grey(5, 42) }
        : { primary: grey(22, 13), secondary: grey(8, 38), disabled: grey(6, 62) },
      divider: dark ? alpha(grey(20, 80), 0.1) : alpha(grey(30, 15), 0.11),
      action: dark
        ? { hover: alpha(t.primary, 0.07), selected: alpha(t.primary, 0.14) }
        : { hover: alpha(t.primary, 0.05), selected: alpha(t.primary, 0.1) },
    },
    typography: {
      fontFamily: ['"Segoe UI Variable Text"', '"Segoe UI"', '-apple-system', 'BlinkMacSystemFont', 'Roboto', 'sans-serif'].join(','),
      fontSize: 13,
      button: { textTransform: 'none', fontWeight: 500 },
      h6: { fontSize: '1rem', fontWeight: 600 },
      subtitle1: { fontSize: '0.95rem', fontWeight: 600 },
      subtitle2: { fontSize: '0.85rem', fontWeight: 600 },
    },
  });
  const { palette } = base;

  return createTheme(base, {
    components: {
      MuiCssBaseline: {
        styleOverrides: {
          body: { userSelect: 'none', overscrollBehavior: 'none' },
          'input, textarea, [contenteditable]': { userSelect: 'text' },
          '*::-webkit-scrollbar': { width: 10, height: 10 },
          '*::-webkit-scrollbar-thumb': {
            backgroundColor: alpha(palette.text.primary, 0.18),
            borderRadius: 5,
            border: `2px solid ${palette.background.paper}`,
          },
          '*::-webkit-scrollbar-thumb:hover': { backgroundColor: alpha(palette.text.primary, 0.32) },
          '*::-webkit-scrollbar-track': { backgroundColor: 'transparent' },
        },
      },
      MuiButtonBase: { defaultProps: { disableRipple: true } },
      MuiButton: {
        defaultProps: { size: 'small', disableElevation: true },
        styleOverrides: { root: { minWidth: 0, padding: '3px 10px', whiteSpace: 'nowrap', flexShrink: 0 }, startIcon: { marginRight: 6 } },
      },
      MuiIconButton: { defaultProps: { size: 'small' }, styleOverrides: { root: { borderRadius: 4, padding: 4 } } },
      MuiToggleButton: { defaultProps: { size: 'small' } },
      MuiTextField: { defaultProps: { size: 'small' } },
      MuiInputBase: { styleOverrides: { root: { fontSize: '0.8125rem' } } },
      MuiFormControl: { defaultProps: { size: 'small' } },
      MuiSelect: { defaultProps: { size: 'small' } },
      MuiAutocomplete: { defaultProps: { size: 'small' } },
      MuiSwitch: { defaultProps: { size: 'small' } },
      MuiCheckbox: { defaultProps: { size: 'small' } },
      MuiRadio: { defaultProps: { size: 'small' } },
      MuiFormControlLabel: { styleOverrides: { root: { marginLeft: -6 }, label: { fontSize: '0.8125rem' } } },
      MuiPaper: {
        defaultProps: { elevation: 0 },
        styleOverrides: { root: { backgroundImage: 'none' }, outlined: { borderColor: palette.divider } },
      },
      MuiAppBar: { defaultProps: { elevation: 0, color: 'default' } },
      MuiToolbar: { styleOverrides: { root: { minHeight: 40, '@media (min-width:600px)': { minHeight: 40 } } } },
      MuiTabs: { styleOverrides: { root: { minHeight: 34 }, indicator: { height: 2 } } },
      MuiTab: { styleOverrides: { root: { minHeight: 34, minWidth: 0, padding: '6px 12px', fontSize: '0.8125rem' } } },
      MuiTable: { defaultProps: { size: 'small' } },
      MuiTableCell: {
        styleOverrides: {
          root: { padding: '4px 8px', borderColor: palette.divider, fontSize: '0.8125rem' },
          head: { fontSize: '0.75rem', fontWeight: 600, color: palette.text.secondary, backgroundColor: palette.background.paper },
        },
      },
      MuiChip: {
        defaultProps: { size: 'small' },
        styleOverrides: { root: { borderRadius: 4, height: 20, fontSize: '0.75rem' }, icon: { fontSize: 14 } },
      },
      MuiTooltip: {
        defaultProps: { enterDelay: 400, disableInteractive: true },
        styleOverrides: { tooltip: { fontSize: '0.75rem', borderRadius: 3, padding: '3px 7px' } },
      },
      MuiDialog: { styleOverrides: { paper: { borderRadius: 6, border: `1px solid ${palette.divider}` } } },
      MuiDialogTitle: { styleOverrides: { root: { fontSize: '0.95rem', fontWeight: 600, padding: '12px 16px' } } },
      MuiDialogContent: { styleOverrides: { root: { padding: '8px 16px' } } },
      MuiDialogActions: { styleOverrides: { root: { padding: '8px 16px 12px' } } },
      MuiAlert: { styleOverrides: { root: { padding: '2px 10px', fontSize: '0.8125rem' } } },
      MuiMenu: { defaultProps: { slotProps: { paper: { variant: 'outlined' } } } },
      MuiMenuItem: { defaultProps: { dense: true } },
      MuiList: { defaultProps: { dense: true } },
      MuiListItemButton: { styleOverrides: { root: { borderRadius: 4 } } },
      MuiLinearProgress: { styleOverrides: { root: { borderRadius: 2 } } },
    },
  });
}

export const lightTheme = appTheme('light');
export const darkTheme = appTheme('dark');
