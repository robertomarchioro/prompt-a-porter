; Hook NSIS per Prompt a Porter — installazione opzionale della CLI `pap`.
;
; `pap.exe` è bundlato come risorsa (finisce in $INSTDIR accanto all'app).
; Qui l'installer CHIEDE se aggiungere $INSTDIR al PATH utente, così `pap`
; diventa richiamabile da terminale; la disinstallazione lo rimuove.
; Solo macro-hook di Tauri (installerHooks), nessun template custom.
;
; ${Using:StrFunc} è idempotente: sicuro anche se il template Tauri
; istanziasse già StrStr/UnStrRep. Compila pulito con makensis 3.09
; (installer + uninstaller) — vedi validazione in fase di sviluppo.

!include "StrFunc.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"

${Using:StrFunc} StrStr
${Using:StrFunc} UnStrRep

!macro NSIS_HOOK_POSTINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION "Vuoi aggiungere lo strumento da riga di comando 'pap' al PATH?$\n$\nCosì potrai eseguire 'pap' dal Prompt dei comandi o da PowerShell." IDNO pap_path_skip
    ReadRegStr $0 HKCU "Environment" "Path"
    ${If} $0 == ""
      WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
    ${Else}
      ${StrStr} $1 "$0" "$INSTDIR"
      ${If} $1 == ""
        WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
      ${EndIf}
    ${EndIf}
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  pap_path_skip:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  ${If} $0 != ""
    ${UnStrRep} $1 "$0" ";$INSTDIR" ""
    ${UnStrRep} $1 "$1" "$INSTDIR;" ""
    ${UnStrRep} $1 "$1" "$INSTDIR" ""
    ${IfNot} $1 == $0
      WriteRegExpandStr HKCU "Environment" "Path" "$1"
      SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
    ${EndIf}
  ${EndIf}
!macroend
