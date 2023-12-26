/* tslint:disable */
/* eslint-disable */
/**
* @param {any} data
* @returns {any}
*/
export function parse_js(data: any): any;

interface TypeScriptSettings {
	decorators?: boolean;
}



type DiagnosticSeverityInput = "Error" | "Warning" | "Information" | "Hint";



interface InputData {
	uri: string;
	file_content: string;
	typescript_settings?: TypeScriptSettings;
	ids_to_check: string[];
	debug?: boolean;
  throw_statement_severity?: DiagnosticSeverityInput;
  function_throw_severity?: DiagnosticSeverityInput;
  call_to_throw_severity?: DiagnosticSeverityInput;
  call_to_imported_throw_severity?: DiagnosticSeverityInput;
  include_try_statement_throws?: boolean;
}



interface ImportedIdentifiers {
	diagnostics: any[];
	id: string;
}



interface ParseResult {
	diagnostics: any[];
	relative_imports: string[];
	throw_ids: string[];
	imported_identifiers_diagnostics: Map<string, ImportedIdentifiers>;
}


