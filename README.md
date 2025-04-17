# HLSL parser

A library written in rust to parse and expose AST from hlsl files. This is a WIP, and the current development plan of this library is as follows:

[x] Read and load HLSL files
[ ] Find #include directives to load all files from disk
[ ] Create a preprocessor tokenizer
  - Should not handle preprocessor directives immediately
  - Allows the parser to define the preprocessor input and cache the files
[ ] Create HLSL parser
[ ] Cache parsed files
[ ] Allow file updates in memory for future integration with language server
[ ] File Updates should not create a full reparse of the AST, but only a subsection

# pipeline
For each file we have the following pipeline to process the files:

1. Read the contents of the file
2. Lex the content to tokens, making a stack of preprocessor definitions, parsing directives 
3. Traverse the lexed tokens 

## Test suite

Almost all files come from the (HlslTools)[https://github.com/tgjones/HlslTools] extension for Visual Studio. This project was inspired by HlslTools as it did not always work fast and perfectly. As the lead developer left the project years ago, we need to develop a new tool that can handle the new HLSL language features.