package main

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"strings"

	extism "github.com/extism/go-sdk"
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
	}
	log.Println("watching at:", path)

	// Look for other directories within the path and watch those too
	dirs := make([]string, 0)
	fs.WalkDir(os.DirFS(path), ".", func(name string, entry os.DirEntry, err error) error {
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
	log.Println("watching dirs:", dirs)

	// Create a collection to store our plug-ins throughout the apps lifetime
	plugins := make(map[string]*extism.Plugin)

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
						continue
					}
				}

				for _, name := range files {
					if strings.HasSuffix(name, ".wasm") && !strings.HasSuffix(event.Name, ".wasm") {
						path := filepath.Join(dir, name)
						pluginManifest := extism.Manifest{
							Wasm: []extism.Wasm{extism.WasmFile{
								Path: path,
							}},
						}

						// load the wasm as an extism plug-in (if cached, use existing plug-in)
						var plugin *extism.Plugin
						if preloaded, ok := plugins[path]; ok {
							plugin = preloaded
						} else {
							plugin, err = extism.NewPlugin(context.Background(), pluginManifest, extism.PluginConfig{}, nil)
							catch(err, fmt.Sprintf("load plugin from wasm: %s", path))
							plugin.SetLogLevel(extism.LogLevelDebug)
							plugin.SetLogger(func(level extism.LogLevel, msg string) {
								log.Printf("[%s] %s", level, msg)
							})
							plugins[path] = plugin
							log.Println("loaded module:", path)
						}

						// read event trigger file
						info, err := os.Stat(event.Name)
						catch(err, "stat trigger file")
						if info.IsDir() {
							continue
						}

						// if the plug-in doesn't want to use the file from the event, skip the
						// event altogether
						_, _, err = plugin.Call("should_handle_file", []byte(event.Name))
						if err != nil {
							// presence of err here indicates to skip the file (avoid copying file)
							fmt.Println("should_handle_file:", err)
							continue
						}

						// create input data to share with plug-in
						eventFileData, err := os.ReadFile(event.Name)
						catch(err, "get target file data")

						eventInput := EventInput{

							EventFileData: base64.StdEncoding.EncodeToString(eventFileData),
							EventFileName: event.Name,
						}
						input, err := json.Marshal(&eventInput)
						catch(err, "serialize event input to json")

						// use input bytes and invoke the plug-in function
						_, output, err := plugin.Call("on_file_write", input)
						catch(err, "calling on_file_write")
						log.Printf(
							"called on_file_write in plugin: %s [%s]\n", name, event.Name,
						)

						// take the output bytes from the plug-in and write them to the trigger file
						if len(output) != 0 {
							var out EventOutput
							err := json.Unmarshal(output, &out)
							catch(err, "unmarshal plug-in output")

							// rather than giving the plug-in access to modify files directly, allow
							// it to give the host an instruction, which the host can follow or not
							b64file := out.OutputFileData
							switch out.Op {
							case "overwrite":
								data, err := base64.StdEncoding.WithPadding(base64.StdPadding).DecodeString(b64file)
								catch(err, "decode output file data for overwrite")
								catch(os.WriteFile(event.Name, data, 0755), "writing output to file")
							case "create":
								data, err := base64.StdEncoding.WithPadding(base64.StdPadding).DecodeString(b64file)
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
