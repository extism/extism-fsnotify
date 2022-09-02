package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/extism/extism"
	"github.com/fsnotify/fsnotify"
)

type EventInput struct {
	EventFileName string            `json:"event_file_name"`
	EventFileData string            `json:"event_file_data"`
	DirEntryFiles map[string]string `json:"dir_entry_files"`
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

	extism.SetLogFile("output.log", "debug")
	fmt.Println("set log file..")

	// Start listening for events.
	go func() {
		for {
			select {
			case event, ok := <-watcher.Events:
				if !ok {
					return
				}
				log.Println("event:", event)

				if event.Op != fsnotify.Create {
					continue
				}

				dir := filepath.Dir(event.Name)
				entries, err := os.ReadDir(dir)
				catch(err, fmt.Sprintf("read dir from %s", event.Name))
				files := make(map[string]string)
				for _, file := range entries {
					if file.IsDir() {
						continue
					}
					data, err := ioutil.ReadFile(event.Name)
					catch(err, "read file to pass to wasm as json")

					fmt.Println(file.Name())
					files[file.Name()] = base64.RawStdEncoding.EncodeToString(data)
				}

				for name := range files {
					fmt.Println(name)
					if strings.HasSuffix(name, ".wasm") {
						module := fmt.Sprintf("%s%s%s", dir, string(filepath.Separator), name)
						wasm, err := os.Open(module)
						catch(err, "open wasm file")

						log.Println("loading module:", module)

						plugin, err := extism.LoadPlugin(wasm, false)
						catch(err, "load plugin from wasm")

						eventFileData, err := os.ReadFile(event.Name)
						catch(err, "get target file data")

						eventInput := EventInput{
							EventFileData: base64.RawStdEncoding.EncodeToString(eventFileData),
							EventFileName: filepath.Base(event.Name),
							DirEntryFiles: files,
						}
						input, err := json.Marshal(&eventInput)
						catch(err, "serialize event input to json")

						output, err := plugin.Call("on_file_write", input)
						catch(err, "calling on_file_write")

						catch(os.WriteFile(event.Name, output, 0755), "writing output to file")
						break
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
