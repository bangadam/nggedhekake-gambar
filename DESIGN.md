# Design System

## Theme

A light, restrained desktop utility for focused image work. The interface should feel quiet and exact under normal office or studio lighting. No dark-tool reflex, AI spectacle, or decorative surfaces.

## Color

- Canvas: `#F7F6F3`
- Primary surface: `#FFFFFF`
- Secondary surface: `#F1F0EC`
- Border: `#E3E0D9`
- Strong text: `#171717`
- Secondary text: `#6D6A64`
- Tertiary text: `#908C86`
- Primary action: `#171717` with white text
- Success background/text: `#EDF3EC` / `#346538`
- Warning background/text: `#FBF3DB` / `#956400`
- Error background/text: `#FDEBEC` / `#9F2F2D`
- Info background/text: `#E1F3FE` / `#1F6C9F`

Semantic color is always paired with text. Accent is reserved for actions, selection, focus, and state.

## Typography

Use the platform UI sans stack through Slint’s default system font. Use one family across the product. Paths and engine details may use the platform monospace family.

- App title: 28px, semibold
- Section heading: 18px, semibold
- Body: 14px, regular, comfortable line height
- Control label: 13px, medium
- Caption/meta: 12px

## Layout

The workspace is a resizable two-column desktop layout:

- Compact left control rail for source selection and settings
- Dominant right canvas for preview and results
- Thin status region at the bottom

Default window: 1120×720. Minimum window: 860×600. Use 8, 12, 16, 24, and 32px spacing steps. Collapse secondary detail before compromising the preview.

## Components

- Buttons use familiar rectangular desktop geometry, 6px radius, explicit focus, and no decorative shadow.
- Inputs use white surfaces, 1px borders, and 8px radius.
- Panels are flat with at most a 1px border and 10px radius.
- Empty states teach the next action without marketing language.
- Errors use a pale semantic surface, concise cause, and next action where known.
- Image canvas uses a neutral background; checkerboard appears only for transparency.

## Motion

Use 120–240ms state transitions only when they explain hover, focus, expansion, or result arrival. Never gate content on animation. Reduced-motion mode removes movement.

## Accessibility

Maintain WCAG AA contrast. Every action is keyboard reachable, focus is visible, error/status text is semantic, and color is never the sole signal. Do not replace standard desktop affordances with inaccessible custom controls.

The fuller product design reference remains in `docs/design-system.md`.
