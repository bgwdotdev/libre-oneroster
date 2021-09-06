-- TODO: review ON DELETE
CREATE TABLE IF NOT EXISTS Category (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text UNIQUE NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS LineItem (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "description" text
    , "assignDate" text NOT NULL --dateTimeOffset
    , "dueDate" text NOT NULL --dateTimeOffset
    , "classSourcedId" text NOT NULL
    , "categorySourcedId" text NOT NULL
    , "academicSessionSourcedId" text NOT NULL
    , "resultValueMin" real NOT NULL
    , "resultValueMax" real NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId) ON DELETE RESTRICT
    , FOREIGN KEY (categorySourcedId) REFERENCES Category (sourcedId) ON DELETE RESTRICT
    , FOREIGN KEY (academicSessionSourcedId) REFERENCES AcademicSessions (sourcedId) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS Results (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "lineItemSourcedId" text NOT NULL
    , "userSourcedId" text NOT NULL
    , "scoreStatusTypeId" integer NOT NULL
    , "score" real NOT NULL
    , "scoreDate" text NOT NULL -- date
    , "comment" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (lineItemSourcedId) REFERENCES LineItem (sourcedId) ON DELETE CASCADE
    , FOREIGN KEY (userSourcedId) REFERENCES users (sourcedId) ON DELETE RESTRICT
    , FOREIGN KEY (scoreStatusTypeId) REFERENCES ScoreStatusType (id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS ScoreStatusType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
);

CREATE VIEW IF NOT EXISTS CategoryJson AS
    SELECT json_object(
        'sourcedId', Category.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Category.dateLastModified
        , 'title', Category.title
    ) AS 'category'
    FROM
        Category
        LEFT JOIN StatusType ON Category.statusTypeId = StatusType.id
    ORDER BY
        Category.sourcedId
;

CREATE VIEW IF NOT EXISTS VwORGetAllCategories AS
    SELECT json_object(
        'categories', json_group_array(json(category))
    ) AS 'categories'
    FROM CategoryJson
;

CREATE VIEW IF NOT EXISTS VwORGetCategory AS
    SELECT json_object(
        'category', json(category)
    ) AS 'category'
    FROM CategoryJson
;

CREATE VIEW IF NOT EXISTS LineItemJson AS
    SELECT json_object(
        'sourcedId', LineItem.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', LineItem.dateLastModified
        , 'title', LineItem.title
        , 'description', LineItem.description
        , 'assignDate', LineItem.assignDate
        , 'dueDate', LineItem.dueDate
        , 'category', json_object(
            'href', 'category/' || LineItem.categorySourcedId
            , 'sourcedId', LineItem.categorySourcedId
            , 'type', 'category'
        )
        , 'class', json_object(
            'href', 'class/' || LineItem.classSourcedId
            , 'sourcedId', LineItem.classSourcedId
            , 'type', 'class'
        )
        , 'gradingPeriod', json_object(
            'href', 'academicSession/' || LineItem.academicSessionSourcedId
            , 'sourcedId', LineItem.academicSessionSourcedId
            , 'type', 'academicSession'
        )
        , 'resultValueMin', LineItem.resultValueMin
        , 'resultValueMax', LineItem.resultValueMax
    ) AS 'lineItem'
    FROM
        LineItem
        LEFT JOIN StatusType ON LineItem.statusTypeId = StatusType.id
    ORDER BY
        LineItem.sourcedId
;

CREATE VIEW IF NOT EXISTS VwORGetAllLineItems AS
    SELECT json_object(
        'lineItems', json_group_array(json(lineItem))
    ) AS 'lineItems'
    FROM LineItemJson
;


CREATE VIEW IF NOT EXISTS VwORGetLineItem AS
    SELECT json_object(
        'lineItem', json(lineItem)
    ) AS 'lineItem'
    FROM LineItemJson
;

CREATE VIEW IF NOT EXISTS ResultsJson AS
    SELECT json_object(
        'sourcedId', Results.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Results.dateLastModified
        , 'lineItem', json_object(
            'href', 'lineItem/' || Results.lineItemSourcedId
            , 'sourcedId', Results.lineItemSourcedId
            , 'type', 'lineItem'
        )
        , 'student', json_object(
            'href', 'user/' || Results.userSourcedId
            , 'sourcedId', Results.userSourcedId
            , 'type', 'user'
        )
        , 'score', Results.score
        , 'scoreStatus', ScoreStatusType.token
        , 'scoreDate', Results.scoreDate
        , 'comment', Results.comment
    ) AS 'result'
    FROM
        Results
        LEFT JOIN StatusType ON Results.statusTypeId = StatusType.id
        LEFT JOIN ScoreStatusType ON Results.scoreStatusTypeId = ScoreStatusType.id
    ORDER BY
        Results.sourcedId
;

CREATE VIEW IF NOT EXISTS VwORGetAllResults AS
    SELECT json_object(
        'results', json_group_array(json(result))
    ) AS 'results'
    FROM ResultsJson
;


CREATE VIEW IF NOT EXISTS VwORGetResult AS
    SELECT json_object(
        'result', json(result)
    ) AS 'result'
    FROM ResultsJson
;
