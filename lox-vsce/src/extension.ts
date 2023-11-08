// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as wasm from '../out/wasm/lox_wasm';

const outputChannel = vscode.window.createOutputChannel("Lox", "lox");

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "lox-vsce" is now active!');

	const diagnostics = vscode.languages.createDiagnosticCollection("lox");
	context.subscriptions.push(diagnostics);

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

	let scanLox = vscode.commands.registerCommand('lox-vsce.scanLox', () => {
		let activeEditor = vscode.window.activeTextEditor;
		if(!activeEditor) {
			return;
		}
		let contents = activeEditor.document.getText();
		const diagnosticCollection : vscode.Diagnostic[] = [];
		wasm.scan(contents, messageAdder(), diagnosticAdder(diagnosticCollection));
		diagnostics.set(activeEditor.document.uri, diagnosticCollection);
	});

	context.subscriptions.push(scanLox);
}

function diagnosticAdder(coll: vscode.Diagnostic[])  {
	return (line: number, column: number, message: string) => {
		coll.push(createDiagnostic(line, message));
	};
}

function messageAdder()  {
	return (message: string) => {
		outputChannel.append(message);
	};
}

function createDiagnostic(lineNo: number, message: string) : vscode.Diagnostic {
	const range = new vscode.Range(lineNo, 1, lineNo, 2);
	const diagnostic = new vscode.Diagnostic(range, message, vscode.DiagnosticSeverity.Warning);
	return diagnostic;
}

// This method is called when your extension is deactivated
export function deactivate() {}
