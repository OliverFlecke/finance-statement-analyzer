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

    interface IArgParserTemplate with
        member s.Usage =
            match s with
            | Filename _ -> "Filename of csv file to analyze"
            | Depth _ -> "Depth of categories to output"

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
    let helper () =
        let indent = 4

        let amount = summerize n

        printf $"{spaces <| indent * d}{n.category |> padr 20}"
        printColor (numColor amount) (amount |> sprintf "%.2f" |> padl 20)
        printfn ""

    match printDepth with
    | None -> helper ()
    | Some maxDepth when maxDepth >= d -> helper ()
    | _ -> ()

let treePerformdWithSort comparer f tree =
    tree.children |> Seq.sortBy comparer |> Seq.iter (performdWithSort comparer f)

tree |> treePerformdWithSort summerize output

let total = summerize tree

printColor (numColor total) (sprintf "\nTotal: %.2f\n" total)
