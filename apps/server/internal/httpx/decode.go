// Package httpx raccoglie piccoli helper HTTP condivisi tra gli handler del
// server (decodifica JSON con limite di dimensione).
package httpx

import (
	"encoding/json"
	"errors"
	"net/http"
)

// DecodeJSONLimited decodifica il body JSON della richiesta applicando un
// limite di dimensione tramite http.MaxBytesReader, per evitare che un
// client (malevolo o buggy) esaurisca memoria/CPU del server inviando
// payload enormi (CWE-400, Uncontrolled Resource Consumption).
//
// Se il body supera maxBytes risponde 413 Payload Too Large. Se il JSON non
// è valido risponde 400 con badRequestBody. Ritorna true solo se la
// decodifica è riuscita: il chiamante deve interrompere l'handler quando
// ritorna false, la risposta di errore è già stata scritta.
func DecodeJSONLimited(w http.ResponseWriter, r *http.Request, maxBytes int64, dst any, badRequestBody string) bool {
	r.Body = http.MaxBytesReader(w, r.Body, maxBytes)

	if err := json.NewDecoder(r.Body).Decode(dst); err != nil {
		var maxBytesErr *http.MaxBytesError
		if errors.As(err, &maxBytesErr) {
			http.Error(w, `{"error":"corpo della richiesta troppo grande"}`, http.StatusRequestEntityTooLarge)
		} else {
			http.Error(w, badRequestBody, http.StatusBadRequest)
		}
		return false
	}
	return true
}
