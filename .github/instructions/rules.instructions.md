---
description: Coding rules for this project. Apply to all Python files, .env files, and config files.
applyTo: "**/*.py,**/.env*,**/config.py"
---

# Rules

* Environment variables must ALWAYS follow this naming structure:
  * PEDGOG_[function e.g.: `USERS`, `RESERVAS`]_[REST_OF_VARIABLE_NAME]
  * all uppercase
  * ENVIRONMENT VARIABLES CALL MUST BE DEFINED CENTRALIZING USING CONFIG FILE
* No "dev_mode" anywhere
* No emojis
* No documentation, except README.MD
* No tests
* No comments in the code
* The fewer lines per file, the BETTER
* The fewer files, the BETTER
* Whenever logic is repeated, use "utils" folder for specific components definition to avoid duplication
* ALWAYS USE CLEANCODE
* ALWAYS USE REACT-ICONS LIBRARY
* USE DAISYUI AND TAILWIND-CSS
* ALWAYS USE THIS PATTERN OF INPUT:
```tsx
<fieldset className="fieldset">
  <legend className="fieldset-legend">What is your name?</legend>
  <input type="text" className="input" placeholder="Type here" />
  <p className="label">Optional</p>
</fieldset>
```