CREATE TABLE "ActionTypes_Lookup" (
	"ActionTypes_LookupID"	INTEGER NOT NULL,
	"ActionTypes_LookupDescription"	TEXT NOT NULL,
	"SupplCategoryLevel1Code"	TEXT,
	"SupplCategoryLevel2Code"	TEXT
)

CREATE TABLE "ApplicationDocs" (
	"ApplicationDocsID"	INTEGER NOT NULL,
	"ApplicationDocsTypeID"	INTEGER NOT NULL,
	"ApplNo"	TEXT NOT NULL,
	"SubmissionType"	TEXT NOT NULL,
	"SubmissionNo"	INTEGER NOT NULL,
	"ApplicationDocsTitle"	TEXT,
	"ApplicationDocsURL"	TEXT,
	"ApplicationDocsDate"	TEXT,
	FOREIGN KEY("ApplNo") REFERENCES "Applications"("ApplNo")
)

CREATE TABLE "Applications" (
	"ApplNo"	TEXT NOT NULL,
	"ApplType"	TEXT NOT NULL,
	"ApplPublicNotes"	TEXT,
	"SponsorName"	TEXT,
	PRIMARY KEY("ApplNo")
)

CREATE TABLE "ApplicationsDocsType_Lookup" (
	"ApplicationDocsType_Lookup_ID"	INTEGER NOT NULL,
	"ApplicationDocsType_Lookup_Description"	TEXT NOT NULL
)

CREATE TABLE "MarketingStatus" (
	"ApplNo"	TEXT NOT NULL,
	"ProductNo"	TEXT NOT NULL,
	"MarketingStatusID"	INTEGER NOT NULL,
	FOREIGN KEY("MarketingStatusID") REFERENCES "MarketingStatus_Lookup",
	FOREIGN KEY("ProductNo") REFERENCES "Products"("ProductNo"),
	FOREIGN KEY("ApplNo") REFERENCES "Applications"("ApplNo")
)

CREATE TABLE "MarketingStatus_Lookup" (
	"MarketingStatusID"	INTEGER NOT NULL,
	"MarketingStatusDescription"	TEXT NOT NULL,
	PRIMARY KEY("MarketingStatusID")
)

CREATE TABLE "Products" (
	"ProductNo"	TEXT NOT NULL,
	"ApplNo"	TEXT NOT NULL,
	"FORM"	TEXT,
	"Strength"	TEXT,
	"ReferenceDrug"	INTEGER,
	"DrugName"	TEXT,
	"ActiveIngredient"	TEXT,
	"ReferenceStandard"	INTEGER,
	FOREIGN KEY("ApplNo") REFERENCES "ApplicationDocs"("ApplNo")
)

CREATE TABLE "SubmissionClass_Lookup" (
	"SubmissionClass_Lookup"	INTEGER NOT NULL,
	"SubmissionClassCode"	TEXT NOT NULL,
	"SubmissionClassCodeDescription"	TEXT
)

CREATE TABLE "SubmissionPropertyType" (
	"ApplNo"	TEXT NOT NULL,
	"SubmissionType"	INTEGER NOT NULL,
	"SubmissionNo"	TEXT NOT NULL,
	"SubmissionPropertyTypeCode"	TEXT NOT NULL,
	"SubmissionPropertyTypeID"	INTEGER
)

CREATE TABLE "Submissions" (
	"ApplNo"	TEXT NOT NULL,
	"SubmissionClassCodeID"	INTEGER,
	"SubmissionType"	TEXT NOT NULL,
	"SubmissionNo"	INTEGER NOT NULL,
	"SubmissionStatus"	INTEGER,
	"SubmissionStatusDate"	TEXT,
	"SubmissionsPublicNotes"	TEXT,
	"ReviewPriority"	TEXT,
	FOREIGN KEY("ApplNo") REFERENCES "Applications"
)

CREATE TABLE "TE" (
	"ApplNo"	TEXT NOT NULL,
	"ProductNo"	TEXT NOT NULL,
	"MarketingStatusID"	INTEGER NOT NULL,
	"TECode"	TEXT NOT NULL,
	FOREIGN KEY("ProductNo") REFERENCES "Products"("ProductNo"),
	FOREIGN KEY("ApplNo") REFERENCES "Applications"("ApplNo")
)