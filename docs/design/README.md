# Design

- [`tokens.css`](tokens.css): the single source of colour, type, spacing and motion values. The app's
  `app/src/styles/tokens.css` is a verbatim copy (a test checks it).
- [`design-system.md`](design-system.md): the design system that explains those values and the components.
- [`typography.md`](typography.md): the reading faces, scale and text-presentation rules.
- [`reader.html`](reader.html): the reader mock-up; open it in any browser.
- [`check-contrast.py`](check-contrast.py): the colour-contrast check that `npm run verify` runs.
