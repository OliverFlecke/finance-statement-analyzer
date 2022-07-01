#r "nuget: FSharp.Data"
#r "nuget: Argu"

open System

module StringHelpers =
    let padl amount (s: string) = s.PadLeft(amount)
    let padr amount (s: string) = s.PadRight(amount)

    let spaces n = new string (' ', n)

    let numColor n =
        if n > 0.0 then
            ConsoleColor.Green
        else
            ConsoleColor.Red


    let printColor color msg =
        Console.ForegroundColor <- color
        printf $"{msg}"
        Console.ForegroundColor <- ConsoleColor.White

module TransactionTree =
    open FSharp.Data

    [<Literal>]
    let Sample = __SOURCE_DIRECTORY__ + "/sample.csv"

    type Transaction = CsvProvider<Sample>


    type Node =
        { category: string
          children: Node list
          items: Transaction.Row list }

    let createNode c =
        { category = c
          children = []
          items = [] }

    let emptyNode = createNode ""

    let mapOrDefault p f d items =
        let mutable found = false

        let transform x =
            if p x then
                found <- true
                f x
            else
                x

        let items' = items |> List.map transform

        if found then
            items'
        else
            items @ [ f d ]

    let rec insert item (cs: string list) (root: Node) =
        match cs with
        | [] -> { root with items = root.items @ [ item ] }
        | c :: cs' ->
            { root with children = mapOrDefault (fun n -> n.category = c) (insert item cs') (createNode c) root.children }

    let insertRow (node: Node) (item: Transaction.Row) =
        insert item (List.ofArray <| item.Category.Split '/') node

    let performd f node =
        let rec helper d node =
            f d node
            node.children |> Seq.iter (helper (d + 1))

        helper 0 node

    let performdWithSort comparer f node =
        let rec helper d node =
            f d node

            node.children
            |> Seq.sortBy comparer
            |> Seq.iter (helper (d + 1))

        helper 0 node

    let rec perform f node = performd (fun _ x -> f x) node

    let printIndented f tree =
        tree.children
        |> Seq.iter (performd (fun d n -> printfn $"{new string ('\t', d)}{f n}"))

    let getValue (row: Transaction.Row) =

        let mutable result = 0.0

        if Double.TryParse(row.``Debit Amount``, &result) then
            -result
        elif Double.TryParse(row.``Credit Amount``, &result) then
            result
        else
            0

    let buildTree (rows: Transaction.Row seq) = rows |> Seq.fold insertRow emptyNode

open StringHelpers
open TransactionTree
open Argu

type Arguments =
    | [<MainCommand; Mandatory>] Filename of path: string
    | Depth of depth: int
    | PrintItems
    | SummerizeItems

    interface IArgParserTemplate with
        member s.Usage =
            match s with
            | Filename _ -> "Filename of csv file to analyze"
            | Depth _ -> "Depth of categories to output"
            | PrintItems -> "Flag whether items should be outputed"
            | SummerizeItems -> "Flag to indicate whether items should by summerized by their description"

let parser =
    ArgumentParser.Create<Arguments>(programName = "Finance analyzer")

let args =
    parser.ParseCommandLine(raiseOnUsage = false)

if args.IsUsageRequested then
    printfn "%s" <| parser.PrintUsage()
    exit 0

let filename =
    System.IO.Path.Join(__SOURCE_DIRECTORY__, args.GetResult Filename)

if IO.File.Exists filename |> not then
    printColor ConsoleColor.Red $"Unable to find file: '{args.GetResult Filename}'"
    exit 1

let data = Transaction.Load(filename)
let tree = buildTree data.Rows

let rec summerize node : float =
    node.children
    |> Seq.map summerize
    |> Seq.append (node.items |> Seq.map getValue)
    |> Seq.sum

let printDepth = args.TryGetResult Depth

let output d n =
    let padding = 26
    let indent = 4
    let formatNumber = sprintf "%.2f" >> padl padding

    let printItems n =
        let print (description, value) =
            printfn $"{spaces (indent * (d + 1))}{description |> padr (padding - indent - 1)} {value |> formatNumber}"

        let mapper =
            match args.TryGetResult SummerizeItems with
            | None -> Seq.map (fun (x: Transaction.Row) -> (x.``Transaction Description``, getValue x))
            | Some _ ->
                Seq.groupBy (fun x -> x.``Transaction Description``)
                >> Seq.map (fun (d, rows) -> (d, rows |> Seq.sumBy getValue))

        n.items |> mapper |> Seq.sortBy snd |> Seq.iter print

    let helper () =

        let amount = summerize n

        printf $"{spaces <| indent * d}{n.category |> padr padding}"
        printColor (numColor amount) (amount |> formatNumber)
        printfn ""

        if args.TryGetResult PrintItems |> Option.isSome then
            printItems n

    match printDepth with
    | None -> helper ()
    | Some maxDepth when maxDepth >= d -> helper ()
    | _ -> ()

let treePerformdWithSort comparer f tree =
    tree.children
    |> Seq.sortBy comparer
    |> Seq.iter (performdWithSort comparer f)

tree |> treePerformdWithSort summerize output

let rec summerizeWithFilter filter node : float =
    node.children
    |> Seq.map (summerizeWithFilter filter)
    |> Seq.append (node.items |> Seq.map getValue |> Seq.filter filter)
    |> Seq.sum

let total = summerize tree
let debits = summerizeWithFilter ((>) 0) tree
let credits = summerizeWithFilter ((<) 0) tree

printColor (numColor debits) (sprintf "\nDebits:   %.2f\n" debits)
printColor (numColor credits) (sprintf "Credits:   %.2f\n" credits)
printColor (numColor total) (sprintf "Total:     %.2f\n" total)
