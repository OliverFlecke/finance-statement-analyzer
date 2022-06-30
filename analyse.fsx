#r "nuget: FSharp.Data"
#r "nuget: Argu"

open System

module StringHelpers =
    let padl amount (s: string) = s.PadLeft(amount)
    let padr amount (s: string) = s.PadRight(amount)

    let spaces n = new string (' ', n)

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

    interface IArgParserTemplate with
        member s.Usage =
            match s with
            | Filename _ -> "Filename of csv file to analyze"

let parser =
    ArgumentParser.Create<Arguments>(programName = "Finance analyzer")

let args =
    parser.ParseCommandLine(raiseOnUsage = false)

if args.IsUsageRequested then
    printfn "%s" <| parser.PrintUsage()
    exit 0

let filename =
    System.IO.Path.Join(__SOURCE_DIRECTORY__, args.GetResult Filename)

let data = Transaction.Load(filename)
let tree = buildTree data.Rows

let rec summerize node : float =
    node.children
    |> Seq.map summerize
    |> Seq.append (node.items |> Seq.map getValue)
    |> Seq.sum

let output d n =
    let indent = 2

    let amount =
        summerize n
        |> sprintf "%.2f"
        |> padl (20 - indent * d)

    printfn $"{spaces <| indent * d}{n.category |> padr 20} {amount}"

tree.children |> Seq.iter (performd output)
