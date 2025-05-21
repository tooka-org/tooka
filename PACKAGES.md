## 📦 Recommended Go Packages for `tooka`

### 1. **CLI Framework**

* **[spf13/cobra](https://github.com/spf13/cobra)**
  *Purpose*: Provides a simple interface to create powerful modern CLI interfaces.
  *Maintenance*: Actively maintained with a large community and widespread adoption.

### 2. **Glob Pattern Parsing**

* **[gobwas/glob](https://github.com/gobwas/glob)**
  *Purpose*: Offers fast and efficient glob pattern matching, suitable for compile-once patterns.
  *Maintenance*: Actively maintained with good performance benchmarks.
  *Usage Example*:

  ```go
  g := glob.MustCompile("*.jpg")
  if g.Match("photo.jpg") {
      // Match found
  }
  ```



### 3. **File System Abstraction**

* **[spf13/afero](https://github.com/spf13/afero)**
  *Purpose*: Provides a filesystem abstraction layer, allowing for easier testing and flexibility.
  *Maintenance*: Well-maintained and widely used in the Go community.
  *Usage Example*:

  ```go
  fs := afero.NewOsFs()
  afero.WriteFile(fs, "/tmp/testfile", []byte("content"), 0644)
  ```



### 4. **Logging**

* **[rs/zerolog](https://github.com/rs/zerolog)**
  *Purpose*: A fast and efficient structured logger for Go.
  *Maintenance*: Actively maintained with a focus on performance.
  *Usage Example*:

  ```go
  log := zerolog.New(os.Stdout).With().Timestamp().Logger()
  log.Info().Msg("Application started")
  ```



* **[natefinch/lumberjack](https://github.com/natefinch/lumberjack)**
  *Purpose*: Provides rolling logs, useful for managing log file sizes.
  *Maintenance*: Actively maintained and commonly used in conjunction with logging libraries.
  *Usage Example*:

  ```go
  log := &lumberjack.Logger{
      Filename:   "/var/log/tooka.log",
      MaxSize:    10, // megabytes
      MaxBackups: 3,
      MaxAge:     28, //days
  }
  ```

### 5. **Terminal Output Styling**

* **[charmbracelet/lipgloss](https://github.com/charmbracelet/lipgloss)**
  *Purpose*: Style terminal output with ease, allowing for colors, borders, and more.
  *Maintenance*: Actively maintained with a growing community.
  *Usage Example*:

  ```go
  style := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("205"))
  fmt.Println(style.Render("Stylish Output"))
  ```

### 6. **YAML Parsing**

* **[go-yaml/yaml](https://github.com/go-yaml/yaml)**
  *Purpose*: YAML support for Go, used for parsing configuration files.
  *Maintenance*: Actively maintained and widely adopted.
  *Usage Example*:

  ```go
  var config Config
  err := yaml.Unmarshal([]byte(data), &config)
  ```

### 7. **EXIF Data Extraction (Optional)**

* **[rwcarlsen/goexif](https://github.com/rwcarlsen/goexif)**
  *Purpose*: Extract EXIF metadata from images, useful for sorting photos by date.
  *Maintenance*: Actively maintained with support for common EXIF tags.
  *Usage Example*:

  ```go
  f, _ := os.Open("image.jpg")
  x, _ := exif.Decode(f)
  dt, _ := x.DateTime()
  ```

---

## 🧩 Additional Considerations

* **Testing**: Utilize Go's built-in `testing` package for unit tests.
* **Configuration Management**: Store user configurations in a YAML file within the user's home directory, using `os.UserHomeDir()` to determine the path.
* **Cross-Platform Compatibility**: Ensure that all file paths and operations are compatible with both Windows and Linux by using Go's `filepath` package.

---

By integrating these packages, you'll have a robust foundation for `tooka`, enabling efficient file sorting, rule management, and user interaction through a clean CLI interface. If you need assistance with specific implementations or further recommendations, feel free to ask!
