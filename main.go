package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/extism/extism"
	"github.com/fsnotify/fsnotify"
)

type EventInput struct {
	EventFileName string `json:"event_file_name"`
	EventFileData string `json:"event_file_data"`
}

type EventOutput struct {
	Op             string `json:"op"`
	OutputFileName string `json:"output_file_name"`
	OutputFileData string `json:"output_file_data"`
}

func main() {
	// Create new watcher.
	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		log.Fatal(err)
	}
	defer watcher.Close()

	// Caller sets the target path
	path := "."
	if len(os.Args) > 2 {
		path = os.Args[2]
		log.Println("configured watch path to:", path)
	}
	log.Println("watching at:", path)

	// Add the path.
	err = watcher.Add(path)
	if err != nil {
		log.Fatal(err)
	}

	// Look for other directories within the path and watch those too
	dirs := make([]string, 0)
	fs.WalkDir(os.DirFS(path), ".", func(name string, entry os.DirEntry, err error) error {
		if name == "." {
			return nil
		}
		if err != nil {
			return err
		}

		if entry.IsDir() {
			dirs = append(dirs, entry.Name())
			err := watcher.Add(filepath.Join(path, name))
			catch(err, fmt.Sprintf("add nested path: %s", entry.Name()))
		}

		return nil
	})
	log.Println("also watching nested dirs:", dirs)

	// Store logs from extism plug-in runtime
	extism.SetLogFile("output.log", "debug")
	log.Println("set log file:", "output.log")

	// Create a collection to store our plug-ins throughout the apps lifetime
	plugins := make(map[string]extism.Plugin)

	// Start listening for events.
	go func() {
		for {
			select {
			case event, ok := <-watcher.Events:
				if !ok {
					return
				}

				if event.Op != fsnotify.Create {
					continue
				}

				// find relevant files and add/remove watcher paths
				dir := filepath.Dir(event.Name)
				entries, err := os.ReadDir(dir)
				catch(err, fmt.Sprintf("read dir from %s", event.Name))
				files := make([]string, 0)
				for _, file := range entries {
					if !file.IsDir() {
						files = append(files, file.Name())
					} else {
						if event.Op&fsnotify.Create == fsnotify.Create {
							msg := fmt.Sprintf("dynamicly add watch: %s", event.Name)
							err := watcher.Add(event.Name)
							catch(err, msg)
							log.Println(msg)
						}

						// TODO: check if removed/renamed paths are auto-unwatched

						continue
					}
				}

				for _, name := range files {
					if strings.HasSuffix(name, ".wasm") && !strings.HasSuffix(event.Name, ".wasm") {
						module := filepath.Join(dir, name)
						wasm, err := os.Open(module)
						catch(err, "open wasm file")

						// load the wasm as an extism plug-in (if cached, use existing plug-in)
						var plugin extism.Plugin
						if preloaded, ok := plugins[module]; ok {
							plugin = preloaded
						} else {
							plugin, err = extism.LoadPlugin(wasm, false)
							catch(err, fmt.Sprintf("load plugin from wasm: %s", module))
							plugins[module] = plugin
							log.Println("loaded module:", module)
						}

						// read event trigger file
						info, err := os.Stat(event.Name)
						catch(err, "stat trigger file")
						if info.IsDir() {
							continue
						}
						eventFileData, err := os.ReadFile(event.Name)
						catch(err, "get target file data")

						// create input data to share with plug-in
						eventInput := EventInput{
							EventFileData: base64.RawStdEncoding.EncodeToString(eventFileData),
							EventFileName: event.Name,
						}
						input, err := json.Marshal(&eventInput)
						catch(err, "serialize event input to json")

						// if the plug-in doesn't want to use the file from the event, skip the
						// event altogether
						output, err := plugin.Call("handle_file", input)
						if err != nil || len(output) == 0 {
							continue
						}

						// use input bytes and invoke the plug-in function
						output, err = plugin.Call("on_file_write", input)
						catch(err, "calling on_file_write")
						log.Println(fmt.Sprintf(
							"called on_file_write in plugin: %s [%s]", name, event.Name,
						))

						// take the output bytes from the plug-in and write them to the trigger file
						if len(output) != 0 {
							var out EventOutput
							err := json.Unmarshal(output, &out)
							catch(err, "unmarshal plug-in output")

							b64file := strings.ReplaceAll(out.OutputFileData, "\n", "")

							switch out.Op {
							case "overwrite":
								data, err := base64.RawStdEncoding.WithPadding(base64.StdPadding).DecodeString(b64file)
								catch(err, "decode output file data for overwrite")
								catch(os.WriteFile(event.Name, data, 0755), "writing output to file")
							case "create":
								data, err := base64.RawStdEncoding.WithPadding(base64.StdPadding).DecodeString(b64file)
								catch(err, "decode output file data for create")

								catch(
									os.WriteFile(out.OutputFileName, data, 0755),
									"create and write to output file",
								)
							}
						}
					}
				}

			case err, ok := <-watcher.Errors:
				if !ok {
					return
				}
				log.Println("error:", err)
			}
		}
	}()

	// Block main goroutine forever.
	<-make(chan struct{})
}

func catch(err error, msg string) {
	if err != nil {
		fmt.Println(err.Error(), msg)
		os.Exit(1)
	}
}
