# Refactor Regression Checklist

Use this checklist after the recent front-end refactor that split `App.vue` and decomposed `useSkillsManager.ts`.

## Quick automated checks

- [ ] `pnpm run typecheck`
- [ ] `pnpm run build:verify`
- [ ] `node "src/composables/constants.test.mjs"`
- [ ] `node "src/composables/utils.test.mjs"`
- [ ] `node "src/composables/useProjectConfig.test.mjs"`
- [ ] `cd src-tauri && cargo check`

## App shell and preferences

- [ ] App starts without blank screen or console crash
- [ ] Theme toggle switches between light and dark
- [ ] Theme persists after reload
- [ ] Language toggle switches between `zh-CN` and `en-US`
- [ ] Language persists after reload
- [ ] Tab switching works for Local / Market / IDE / Projects / Settings

## Local tab

- [ ] Local skills list renders
- [ ] Refresh action updates local list without error toast
- [ ] Import dialog opens
- [ ] Importing one valid skill shows success and refreshes list
- [ ] Opening a skill directory works for an existing path
- [ ] Manage Versions action opens version manager modal

## Market tab

- [ ] Search input triggers search
- [ ] Initial results render
- [ ] Load more works when more results exist
- [ ] Download queue appears after clicking download
- [ ] Queue item transitions through pending / downloading / done or error
- [ ] Update action still works for already-downloaded skill
- [ ] Market config save still updates enabled market state

## IDE tab

- [ ] IDE filter changes the displayed installed-skill list
- [ ] Add Custom IDE validates empty values and invalid path input
- [ ] Add Custom IDE succeeds with a valid custom path
- [ ] Remove Custom IDE removes the custom entry
- [ ] Adopt single unmanaged IDE skill still works
- [ ] Batch adopt still works
- [ ] Single uninstall modal opens and confirms correctly
- [ ] Batch uninstall modal opens and confirms correctly

## Projects tab

- [ ] Add Project modal opens and saves a project
- [ ] Selecting a project updates the panel correctly
- [ ] Configure Project modal opens and saves IDE targets
- [ ] Export Skills scans project and opens project skill import modal when skills exist
- [ ] Import Skills opens clone-to-project modal
- [ ] Clone selected local skills into project succeeds
- [ ] Project skill snapshots refresh without user-visible breakage
- [ ] No runaway polling or repeated duplicate refresh behavior is observed while staying on Projects tab

## Conflict handling

- [ ] Conflict modal opens for a conflicting imported project skill
- [ ] Keep / overwrite / coexist actions resolve and close modal
- [ ] Local skills refresh after conflict resolution
- [ ] Project scan refreshes after conflict resolution

## Version management

- [ ] Version manager modal opens from a local skill
- [ ] Compare versions opens diff modal
- [ ] Create version from a selected path still works
- [ ] Pick project for import source loads project skills
- [ ] Rename version works
- [ ] Delete version works with supported strategies
- [ ] Set default version works
- [ ] Create variant works
- [ ] Update variant works
- [ ] Delete variant works

## Error-handling spot checks

- [ ] Invalid or unexpected local-scan response surfaces a useful error instead of silently failing
- [ ] Partial uninstall surfaces partial-failure feedback
- [ ] Partial adopt surfaces partial-failure feedback
- [ ] Partial import surfaces failure count feedback

## Exit criteria

The refactor is considered safe to ship when:

- [ ] Automated checks pass
- [ ] Core manual flows above pass on a local Tauri run
- [ ] No new console error pattern appears during the exercised flows
- [ ] No modal becomes stuck open or loses state unexpectedly
