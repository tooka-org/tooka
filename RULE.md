# 📁 Tooka Rule Schema (v2)

Tooka rules are defined in YAML files using a flat, readable structure.
Each rule contains `id`, `name`, `enabled`, `priority`, `description`, `when` conditions, and `then` actions.

---

## 🆔 Top-Level Fields

| Field         | Type    | Required | Description                              |
| ------------- | ------- | -------- | ---------------------------------------- |
| `id`          | string  | ✅        | Unique identifier for the rule.          |
| `name`        | string  | ✅        | Human-readable name of the rule.         |
| `enabled`     | boolean | ✅        | Whether the rule is active.              |
| `priority`    | integer | ✅        | The priority when mathcing, default=1    |
| `description` | string  | ✅        | What this rule does.                     |
| `when`        | list    | ✅        | List of conditions to match files.       |
| `then`        | list    | ✅        | List of actions to perform when matched. |

---

## 🔍 `when`: Match Conditions

A **list of conditions** to evaluate against each file.
Supports a special condition `any: true|false` to control logic mode.

### Structure

```yaml
when:
  - any: false               # default is false (AND logic)
  - extension: ["jpg", "jpeg"]
  - path: "**/holiday/**"
  - metadata.exif_date: true
  - size_kb: ">100"
  - mime_type: "image/jpeg"
  - filename: "~.*holiday.*"
  - is_symlink: false
```

### Supported Conditions

| Field                | Type           | Description                                                   |
| -------------------- | -------------- | ------------------------------------------------------------- |
| `any`                | boolean        | If `true`, use OR logic; otherwise AND (default).             |
| `extension`          | list or string | Match one or more file extensions (no dot, e.g. `"jpg"`).     |
| `path`               | string (glob)  | Match full path using glob (`**/*.jpg`, `**/folder/**`).      |
| `filename`           | string (regex) | Match filename using regex (prefix with `~`).                 |
| `size_kb`            | string         | Use operators: `">100"`, `"<2048"`, `"==512"` (in kilobytes). |
| `mime_type`          | string         | Exact match (e.g., `"image/png"`).                            |
| `metadata.exif_date` | boolean        | `true` if EXIF date metadata is present.                      |
| `is_symlink`         | boolean        | Match symlink status.                                         |

---

## ⚙️ `then`: Actions

A **list of actions** to perform when a file matches the conditions.

### Actions

---

### ➡️ `move`

| Field                | Type    | Description                                      |
| -------------------- | ------- | ------------------------------------------------ |
| `to`                 | string  | Destination directory (absolute or relative).    |
| `preserve_structure` | boolean | If `true`, keeps relative path from source root. |

```yaml
- move:
    to: "/archive/photos/"
    preserve_structure: true
```

---

### 📋 `copy`

Same as `move`, but copies the file instead of moving.

```yaml
- copy:
    to: "/backup/"
    preserve_structure: false
```

---

### ❌ `delete`

| Field   | Type    | Description                           |
| ------- | ------- | ------------------------------------- |
| `trash` | boolean | If `true`, move to trash/recycle bin. |

```yaml
- delete:
    trash: true
```

---

### 📝 `rename`

| Field      | Type   | Description                                              |
| ---------- | ------ | -------------------------------------------------------- |
| `to`       | string | Template for renaming, supports Jinja-style expressions. |

You can use metadata in the name:

```yaml
- rename:
    to: "{{metadata.exif_date|date:%Y-%m-%d}}-{{filename}}"
```

---

### ⏭️ `skip`

Skips further processing for this file.

```yaml
- skip
```

---

## ✅ Example Full Rule

```yaml
id: organize_photos
name: "Organize holiday photos"
enabled: true
description: "Move JPEGs with EXIF data to archive folder"

when:
  - any: false
  - extension: ["jpg", "jpeg"]
  - path: "**/holiday/**"
  - metadata.exif_date: true
  - size_kb: ">100"

then:
  - move:
      to: "/archive/photos/"
      preserve_structure: true
  - rename:
      to: "{{metadata.exif_date|date:%Y-%m-%d}}-{{filename}}"
```

---

## 🛠 Notes

* All paths should be valid for your platform (Unix, Windows).
* `preserve_structure` builds the destination path relative to the rule's base directory.
* `any` defaults to `false`, meaning all conditions must match (AND).
* Files are processed one-by-one; rule chaining is implicit by list order.
