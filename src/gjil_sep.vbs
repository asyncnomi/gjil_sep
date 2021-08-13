If WScript.Arguments(0) = "getPath" Then
  Call getPath(WScript.Arguments(1))
ElseIf WScript.Arguments(0) = "rename" Then
  Call rename(WScript.Arguments(1))
ElseIf WScript.Arguments(0) = "macro" Then
  Call macro(WScript.Arguments(1), WScript.Arguments(2), WScript.Arguments(3))
ElseIf WScript.Arguments(0) = "error" Then
  MsgBox "Erreur d'execution"
ElseIf WScript.Arguments(0) = "end" Then
  MsgBox "Execution terminee"
End If

Sub macro(doc, name, path)
  Set objFileToWrite = CreateObject("Scripting.FileSystemObject").OpenTextFile(path & "\log_macro.txt",2,true)

  Set oWord = CreateObject("Word.Application")
  oWord.Visible = False
  oWord.Documents.Open doc

  On Error Resume Next
  oWord.Run name
  If Err.Number <> 0 Then
    Err.Clear
    objFileToWrite.WriteLine("FAILED")
  Else
    objFileToWrite.WriteLine("SUCCESS")
  End If

  objFileToWrite.Close
  Set objFileToWrite = Nothing
  oWord.Quit
  Set oWord = Nothing
End Sub

Sub rename(path)
  ReadExcelFile = Null

  Dim objExcel, objSheet, objCells

  On Error Resume Next
  Set objExcel = CreateObject("Excel.Application")
  objExcel.Visible = False

  On Error Resume Next
  Call objExcel.Workbooks.Open(path & "\MACROS_GJIL_outil_etat_renom_aquitaine.xlsm", False, True)

  objExcel.Run "recup_noms"
End Sub

Sub getPath(path)
  Set objFileToWrite = CreateObject("Scripting.FileSystemObject").OpenTextFile(path & "\config.txt",2,true)
  objFileToWrite.WriteLine("//This file is automaticaly created, pls do not alter it")
  objFileToWrite.WriteLine("|SEP|")


  ' Local variable declarations
  Dim objExcel, objSheet, objCells

  ' Default return value
  ReadExcelFile = Null
  ' Create the Excel object
  On Error Resume Next
  Set objExcel = CreateObject("Excel.Application")
  objExcel.Visible = False

  ' Open the document as read-only
  On Error Resume Next
  Call objExcel.Workbooks.Open(path & "\MACROS_GJIL_outil_etat_renom_aquitaine.xlsm", False, True)

  Set objSheet = objExcel.ActiveWorkbook.Worksheets(2)
  input = objExcel.Cells(10,2).Value
  tmp = objExcel.Cells(11,2).Value
  objFileToWrite.WriteLine(input)
  objFileToWrite.WriteLine("|SEP|")
  objFileToWrite.WriteLine(tmp)
  objFileToWrite.WriteLine("|SEP|")
  output = objExcel.Cells(14,2).Value

  ' Close the workbook without saving
  Call objExcel.ActiveWorkbook.Close(False)

  ' Quit Excel
  objExcel.Application.Quit

  objFileToWrite.Close
  Set objFileToWrite = Nothing
End Sub
