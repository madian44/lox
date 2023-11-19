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

function addScanSelectedLoxCommand(context: vscode.ExtensionContext, diagnostics: vscode.DiagnosticCollection) {

	defineCommand(context, "lox-vsce.scanSelectedLox", () => {
		const activeEditor = vscode.window.activeTextEditor;
		if(!activeEditor) {
			return;
		}
		const selection = activeEditor.selection;
		if( !selection || selection.isEmpty) {
			return;
		}
		const selectionRange = new vscode.Range(selection.start.line, selection.start.character, selection.end.line, selection.end.character);
		const contents = activeEditor.document.getText(selectionRange);
		const diagnosticCollection : vscode.Diagnostic[] = [];
		wasm.scan(contents, messageAdder(), diagnosticAdder(diagnosticCollection));
		diagnostics.set(activeEditor.document.uri, diagnosticCollection);
	});
}

function defineCommand(context: vscode.ExtensionContext, commandName: string, callback: () => void) {
	let command = vscode.commands.registerCommand(commandName, callback);
	context.subscriptions.push(command);
}

function diagnosticAdder(coll: vscode.Diagnostic[])  {
	return (start: FileLocation, end: FileLocation, message: string) => {
		coll.push(createDiagnostic(start, end, message));
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
