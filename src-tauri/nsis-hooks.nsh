; NSIS-хуки Larpscord: останавливаем запущенное приложение перед обновлением,
; чтобы не плодились дубли в трее.

!macro NSIS_HOOK_PREINSTALL
  ; Закрываем работающий Larpscord перед копированием файлов.
  nsExec::Exec 'taskkill /F /IM "larpscord.exe" /T'
  Pop $0
  Sleep 700
!macroend

!macro NSIS_HOOK_POSTINSTALL
!macroend
