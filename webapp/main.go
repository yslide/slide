package main

import (
	"html/template"
	"net/http"
)

func buildSlide() {
	// Clone, build slide.
	// This should run in a green thread every hour or so?
	// Builds will be incremental after every pull. Built artifacts should be
	// tagged with the appropriate commit, which we can propagate to the UI.
}

func execSlide() {
	// Handle a slide request
}

func main() {
	tmpl := template.Must(template.ParseFiles("forms.html"))

	http.HandleFunc("/execslide", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			tmpl.Execute(w, nil)
			return
		}

		tmpl.Execute(w, struct {
			Success  bool
			Response string
		}{true, "dummy"})

	})
	http.ListenAndServe(":8080", nil)
}
