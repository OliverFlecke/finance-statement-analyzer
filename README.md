# Finance transaction analyzer

Small simple script/application to analyze a csv file with transactions.
Used to summarize in witch categories and places money is being spent.

## Format

The `sample.csv` contains the default headers in the supported csv files.
Note that currently only the following columns are being used for the analysis:

- Transaction Date
- Transaction Description
- Debit Amount
- Credit Amount
- Category

Nested categories are supported and can be defined using `/` as a seperator, e.g. a category for restaurants could be `Food/Restaurant`, which would mean it would be summarized within both the `Food` and the `Resuturant` category.
