# Nggedhekaké Gambar Design System

## 1. Design Direction

Nggedhekaké Gambar should feel like a **native utility for serious image work**, not a flashy AI toy. The visual tone is calm, quiet, and deliberate. The product identity should communicate precision, local control, and technical confidence.

Reference posture:
- native desktop utility first
- editorial clarity second
- AI branding distant third

No gradients. No neon. No glassmorphism. No oversized hype copy.

## 2. Product Personality

- **Quiet**: interface does not shout
- **Precise**: settings read like controls, not marketing cards
- **Local**: files, paths, jobs, models are tangible system objects
- **Fast**: visual hierarchy should imply speed and low overhead
- **Trustworthy**: destructive actions and engine failures are explicit

## 3. Design Principles

### 3.1 One-screen primary workflow
The default screen must support the main task without navigation sprawl:
- choose source
- choose model
- choose output options
- run
- inspect result

### 3.2 Advanced only when needed
Most users should not see technical complexity until they ask for it. Advanced controls live behind a deliberate disclosure pattern.

### 3.3 File-system honesty
Show real filenames, paths, output directories, and engine status. Do not hide system truth behind vague labels.

### 3.4 Strong state visibility
The app must make idle, validating, processing, resizing, success, failure, and cancelled states obvious.

### 3.5 Fewer surfaces, better surfaces
Prefer one strong panel over many floating cards. Prefer one results area over modal clutter.

## 4. Visual Language

### 4.1 Color palette
Base palette:
- Canvas: `#F7F6F3`
- Primary surface: `#FFFFFF`
- Secondary surface: `#F1F0EC`
- Border: `#E3E0D9`
- Strong text: `#171717`
- Secondary text: `#6D6A64`
- Tertiary text: `#908C86`

Semantic colors:
- Success bg: `#EDF3EC`
- Success text: `#346538`
- Warning bg: `#FBF3DB`
- Warning text: `#956400`
- Error bg: `#FDEBEC`
- Error text: `#9F2F2D`
- Info bg: `#E1F3FE`
- Info text: `#1F6C9F`

Accent usage should be scarce. Most of the UI stays monochrome.

### 4.2 Typography
Recommended stack:
- UI sans: `SF Pro Display`, `Geist Sans`, `Helvetica Neue`, `Arial`, sans-serif
- Mono: `SF Mono`, `Geist Mono`, `JetBrains Mono`, monospace

Type scale:
- App title: 28 / semibold / tracking -0.03em
- Section heading: 18 / semibold
- Control label: 13 / medium
- Body: 14 / regular / line-height 1.5
- Caption/meta: 12 / regular
- Mono path/engine output: 12 / mono

Typography rules:
- no pure black on pure white where avoidable
- path strings and engine details use mono
- labels short, factual, no marketing phrasing

## 5. Layout System

### 5.1 App shell
Desktop shell divided into 3 zones:
- **Header bar**: app identity, engine status, settings entry
- **Main workspace**: source + controls + preview/result
- **Footer/status rail**: active job status, output path, logs shortcut

### 5.2 Main workspace grid
Recommended v1 desktop grid:
- left rail: source selection and core settings
- center canvas: preview / compare / result surface
- right rail: model details, job status, advanced settings or recent outputs

If space is tight, collapse right rail into drawers/sections rather than new windows.

### 5.3 Spacing scale
- 4 px hairline spacing only inside compact micro-controls
- 8 px compact gaps
- 12 px default form gaps
- 16 px component padding
- 24 px panel padding
- 32 px section spacing

## 6. Components

### 6.1 Buttons
Primary button:
- background `#171717`
- text white
- radius 6px
- no shadow
- hover darkens slightly
- disabled state lowers contrast, no fake glow

Secondary button:
- surface white or secondary surface
- 1px border
- dark text

Destructive button:
- pale red background with red text for soft actions
- strong destructive confirmation uses bordered red emphasis

### 6.2 Input surfaces
Text inputs, select controls, and path displays:
- 1px solid border `#E3E0D9`
- radius 8px
- background white
- focus ring minimal, 1px or 2px outline in muted dark tone

### 6.3 Segmented controls
Use for mode toggles like:
- single / batch
- 2x / 4x / custom width
- before / after / compare

Keep segments compact and rectangular, not pill-heavy.

### 6.4 Cards and panels
- flat surfaces only
- 1px border
- radius 10-12px max
- no large shadows
- one panel can be tinted secondary surface to anchor information hierarchy

### 6.5 Status badges
Use only when they encode state:
- Idle
- Ready
- Running
- Resizing
- Done
- Failed
- Cancelled

Style:
- small uppercase or compact title case
- semantic muted backgrounds
- never as decoration

### 6.6 Progress indicator
Need both:
- numeric/linear progress if engine emits parseable progress
- textual phase label when only coarse state is known

Progress area should also show:
- model used
- scale used
- source filename
- elapsed time if practical

### 6.7 Preview/result surface
Preview zone is the emotional center of the app.
It should support:
- input preview before run
- result preview after run
- side-by-side compare or swipe compare
- open file / open folder actions

Background behind images should be neutral and non-distracting.
Use checkerboard only when transparency matters.

### 6.8 Log panel
A collapsible bottom drawer or side sheet for engine logs:
- mono text
- selectable
- copyable
- filterable by warnings/errors in future versions

## 7. Copy System

Tone:
- short
- direct
- operational
- no AI hype

Examples:
- Good: `Choose image`, `Batch folder`, `Model`, `Tile size`, `Output format`
- Bad: `Enhance your creative workflow`, `Unleash next-gen clarity`

Status copy:
- `Validating model files`
- `Starting engine`
- `Upscaling`
- `Resizing output`
- `Done`
- `Cancelled`
- `Vulkan runtime not available`

Error copy rules:
- say what failed
- say where if path-related
- say next action if known

## 8. Motion

Motion should be nearly invisible.

Allowed:
- 120–180ms hover transitions
- 180–240ms panel expand/collapse
- 200ms status badge/color transitions
- subtle fade/translate for result appearance

Avoid:
- springy UI
- parallax
- decorative loading animations
- endless shimmer effects

## 9. Accessibility

- all controls keyboard reachable
- visible focus state always present
- semantic color cannot be sole signal; pair with text/icon
- previews need alt/state labels where practical
- error messages announced in accessible status regions
- body text minimum 14px in most places

## 10. Iconography

Use thicker, technical-feeling icons. Avoid thin-line generic startup icon sets.
Preferred traits:
- simple geometry
- consistent stroke weight
- filesystem and processing metaphors

Core icons needed:
- image/file
- folder
- play/start
- stop/cancel
- settings/sliders
- compare/split view
- warning/error
- GPU/engine status

## 11. Core Screens

### 11.1 Home / Workspace
Contains:
- source selector
- mode toggle
- model selector
- core output controls
- run button
- preview area
- status area

### 11.2 Advanced Settings
Contains:
- tile size
- compression
- TTA
- metadata copy
- overwrite behavior
- custom model directory
- diagnostics entry

### 11.3 Result State
Contains:
- preview compare
- open result
- open containing folder
- rerun with changes
- duplicate settings into new run

### 11.4 Diagnostics / Engine Status
Contains:
- engine binary found / missing
- models found / missing
- Vulkan capability summary
- app version
- engine version if detectable
- copy diagnostic report

## 12. Brand Notes

The name **Nggedhekaké Gambar** is strong already. The brand should not be over-designed.

Logo direction:
- simple geometric enlargement motif
- square frame + expanded square or crop marks
- monochrome first

Wordmark behavior:
- use title in header
- optionally short app mark in compact contexts

## 13. V1 Rules

- one window
- no tab maze
- no fake dashboard
- no onboarding carousel
- no cloud upsell
- no community feed
- no unnecessary animations
- no more than one primary CTA on the main screen

## 14. Design QA Checklist

Before implementation, every screen should pass:
- Is the primary action obvious in under 3 seconds?
- Can a user start a job without reading a paragraph?
- Are advanced controls hidden until needed?
- Does the screen still look calm during failure states?
- Are paths, models, and outputs presented clearly?
- Does this feel lighter and more native than an Electron clone?
