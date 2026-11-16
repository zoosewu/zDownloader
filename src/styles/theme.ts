// Material UI theme: a dense desktop-application look (small radii, compact controls, no
// ripples or shadows), in light and dark. Colors follow the app icon (teal to emerald).
import { createTheme, alpha, type PaletteMode, type Theme } from '@mui/material/styles';

function appTheme(mode: PaletteMode): Theme {
  const dark = mode === 'dark';
  const base = createTheme({
    spacing: 6,
    shape: { borderRadius: 4 },
    palette: {
      mode,
      primary: { main: dark ? '#34d399' : '#0f766e' },
      secondary: { main: dark ? '#5eead4' : '#0d9488' },
      background: dark ? { default: '#17181a', paper: '#1f2023' } : { default: '#f3f4f5', paper: '#ffffff' },
      divider: dark ? 'rgba(255,255,255,0.09)' : 'rgba(0,0,0,0.10)',
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
