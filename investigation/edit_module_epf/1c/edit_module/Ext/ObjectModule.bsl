
Procedure ShowTest() Export
	
	Cats = Metadata.Catalogs.Count();
	Docs = Metadata.Documents.Count();
	
	Message(Cats + Docs);
	
EndProcedure