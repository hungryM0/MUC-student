import { makeStyles, shorthands, tokens } from '@fluentui/react-components';

export const useShellStyles = makeStyles({
  provider: {
    width: '100vw',
    height: '100vh',
    background: 'transparent !important'
  },
  frame: {
    display: 'grid',
    gridTemplateColumns: '264px minmax(0, 1fr)',
    width: '100vw',
    height: '100vh',
    overflow: 'hidden',
    background: 'transparent',
    color: tokens.colorNeutralForeground1,
    userSelect: 'none'
  },
  sidebar: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    padding: '16px 10px',
    borderRight: `1px solid ${tokens.colorNeutralStroke3}`,
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground2) 54%, transparent)',
    backdropFilter: 'blur(22px)'
  },
  brand: {
    display: 'flex',
    alignItems: 'center',
    minHeight: '44px',
    padding: '0 12px 8px',
    fontSize: tokens.fontSizeBase500,
    fontWeight: tokens.fontWeightSemibold
  },
  navButton: {
    justifyContent: 'flex-start',
    minHeight: '42px',
    borderRadius: tokens.borderRadiusMedium,
    fontSize: tokens.fontSizeBase400
  },
  navButtonActive: {
    position: 'relative',
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground1) 74%, transparent)',
    ':before': {
      position: 'absolute',
      left: '0',
      width: '3px',
      height: '18px',
      borderRadius: tokens.borderRadiusCircular,
      backgroundColor: tokens.colorBrandForeground1,
      content: '""'
    }
  },
  sidebarSpacer: {
    flex: 1
  },
  page: {
    display: 'flex',
    flexDirection: 'column',
    minWidth: 0,
    minHeight: 0,
    gap: '20px',
    padding: '32px 40px 36px',
    overflow: 'hidden'
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '20px',
    minHeight: '48px'
  },
  title: {
    margin: 0,
    fontSize: tokens.fontSizeHero800,
    fontWeight: tokens.fontWeightSemibold,
    letterSpacing: 0
  },
  toolbar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'flex-end',
    flexWrap: 'wrap',
    gap: '10px',
    minWidth: 0
  },
  content: {
    display: 'flex',
    flexDirection: 'column',
    gap: '18px',
    flex: 1,
    minHeight: 0,
    minWidth: 0
  },
  stack: {
    display: 'flex',
    flexDirection: 'column',
    gap: '18px',
    minHeight: 0
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    minWidth: 0
  },
  splitRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '16px',
    minWidth: 0
  },
  scrollArea: {
    flex: 1,
    minHeight: 0,
    overflow: 'auto',
    paddingRight: '6px'
  },
  card: {
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke3),
    borderRadius: tokens.borderRadiusXLarge,
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground2) 76%, transparent)',
    boxShadow: tokens.shadow4
  },
  lightSafeCard: {
    '@media (prefers-color-scheme: light)': {
      backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground1) 90%, transparent)'
    }
  },
  compactCard: {
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke3),
    borderRadius: tokens.borderRadiusLarge,
    padding: '16px',
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground2) 82%, transparent)'
  },
  muted: {
    color: tokens.colorNeutralForeground3
  },
  success: {
    color: tokens.colorPaletteGreenForeground1
  },
  danger: {
    color: tokens.colorPaletteRedForeground1
  },
  brandText: {
    color: tokens.colorBrandForeground1
  },
  statusDot: {
    width: '8px',
    height: '8px',
    flex: '0 0 auto',
    borderRadius: tokens.borderRadiusCircular
  }
});

export const iconSize = { fontSize: '20px' };
