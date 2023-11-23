// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as wasm from '../out/wasm/lox_wasm';
import {FileLocation} from "./lox";

const outputChannel = vscode.window.createOutputChannel("Lox", "lox");

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "lox-vsce" is now active!');


	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	let helloLox = vscode.commands.registerCommand('lox-vsce.helloLox', () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
        wasm.greet();
		vscode.window.showInformationMessage('Hello World from lox-vsce!');
	});

	context.subscriptions.push(helloLox);

	const diagnostics = vscode.languages.createDiagnosticCollection("lox");
	context.subscriptions.push(diagnostics);

	addScanLoxCommand(context, diagnostics);
	addScanSelectedLoxCommand(context, diagnostics);
	addParseSelectedLoxCommand(context, diagnostics);
}

function addScanLoxCommand(context: vscode.ExtensionContext, diagnostics: vscode.DiagnosticCollection) {

	defineCommand(context, "lox-vsce.scanLox", () => {
		const activeEditor = vscode.window.activeTextEditor;
		if(!activeEditor) {
			return;
		}
		const contents = activeEditor.document.getText();
		const diagnosticCollection : vscode.Diagnostic[] = [];
		wasm.scan(contents, messageAdder(), diagnosticAdder(diagnosticCollection));
		diagnostics.set(activeEditor.document.uri, diagnosticCollection);
	});
}

function getSelectedText(activeEditor: vscode.TextEditor) : [string, vscode.Selection] {
	const selection = activeEditor.selection;
	if( !selection || selection.isEmpty) {
		return ["", selection];
	}
	const selectionRange = new vscode.Range(selection.start.line, selection.start.character, selection.end.line, selection.end.character);
	return [activeEditor.document.getText(selectionRange), selection];
}

function addScanSelectedLoxCommand(context: vscode.ExtensionContext, diagnostics: vscode.DiagnosticCollection) {

	defineCommand(context, "lox-vsce.scanSelectedLox", () => {
		const activeEditor = vscode.window.activeTextEditor;
		if(!activeEditor) {
			return;
		}

		const [contents , selection] = getSelectedText(activeEditor);
		if( !selection || selection.isEmpty) {
			return;
		}

		const diagnosticCollection : vscode.Diagnostic[] = [];
		wasm.scan(contents, messageAdder(), diagnosticAdder(diagnosticCollection, selection.start.line, selection.start.character));
		diagnostics.set(activeEditor.document.uri, diagnosticCollection);
	});
}

function addParseSelectedLoxCommand(context: vscode.ExtensionContext, diagnostics: vscode.DiagnosticCollection) {

	defineCommand(context, "lox-vsce.parseSelectedLox", () => {
		const activeEditor = vscode.window.activeTextEditor;
		if(!activeEditor) {
			return;
		}

		const [contents , selection] = getSelectedText(activeEditor);
		if( !selection || selection.isEmpty) {
			return;
		}
		const diagnosticCollection : vscode.Diagnostic[] = [];
		wasm.parse(contents, messageAdder(), diagnosticAdder(diagnosticCollection, selection.start.line, selection.start.character));
		diagnostics.set(activeEditor.document.uri, diagnosticCollection);
	});
}

function defineCommand(context: vscode.ExtensionContext, commandName: string, callback: () => void) {
	let command = vscode.commands.registerCommand(commandName, callback);
	context.subscriptions.push(command);
}

function diagnosticAdder(collection: vscode.Diagnostic[], startLine: number = 0, startCharacter: number = 0)  {
	let firstDiagnostic = true;

	return (start: FileLocation, end: FileLocation, message: string) => {
		start.line_number += startLine;
		end.line_number += startLine;
		if(firstDiagnostic && startCharacter !== 0) {
			start.line_offset += startCharacter;
			if(start.line_number === end.line_number) {
				end.line_offset += startCharacter;
			}
		}
		firstDiagnostic = false;
		collection.push(createDiagnostic(start, end, message));
	};
}

function messageAdder()  {
	return (message: string) => {
		outputChannel.appendLine(message);
	};
}

function createDiagnostic(start: FileLocation, end: FileLocation, message: string) : vscode.Diagnostic {
	const range = new vscode.Range(start.line_number, start.line_offset, end.line_number, end.line_offset);
	const diagnostic = new vscode.Diagnostic(range, message, vscode.DiagnosticSeverity.Warning);
	return diagnostic;
}

// This method is called when your extension is deactivated
export function deactivate() {}
