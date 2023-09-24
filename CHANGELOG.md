# Changelog

## Unreleased

## 2023-09-23 v0.1.7

- Add arguments to `issue list`
  - `team`
  - `noteam`
  - `noproject`
- Fix auth issue, API removed `Bearer`.

## 2023-09-11 v0.1.6

- Add `template evaluate`

## 2023-07-19 v0.1.5

- Add arguments to `issue create`
  - `title`
  - `description`
  - `team`
  - `noproject`

## 2023-07-12 v0.1.4

- Improve issue selection
- Add comments to issues

## 2023-07-10 v0.1.3

- Add `issue list`
- Add `org` argument to `issue` sub commands
- Show children count
- Add `select` flag to `view`

## 2023-07-07 v0.1.2

- Only show URL when issue is updated
- Add `org list`
- Fix VERSION_URL
- Put `None` first when selecting project
- Sort select lists alphabetically

## 2023-07-04 v0.1.1

- Fix website link in `Cargo.toml`
- Use `md` extension when creating temp file for editor
- Fix GQL query for `issue create`

## 2023-07-04 v0.1.0

- Add skeleton of app
- Add `issue create`
- Add `org add`
- Add `org remove`
- Add badges to `README.md`
- Add `issue view`
- Add `issue edit`