#r "nuget: FSharp.Data"

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

    let rec insert (root: Node) item (cs: string list) =
        match cs with
        | [] -> { root with items = root.items @ [ item ] }
        | c :: cs' ->
            if List.exists (fun x -> x.category = c) root.children then
                let insertIfCategory =
                    (fun n ->
                        if n.category = c then
                            insert n item cs'
                        else
                            n)

                { root with children = root.children |> List.map insertIfCategory }
            else
                { root with children = root.children @ [ insert (createNode c) item cs' ] }

    let insertRow (root: Node) (item: Transaction.Row) =
        insert root item
        <| (List.ofArray <| item.Category.Split '/')

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

printfn ("Starting analyze...")

let args = Environment.GetCommandLineArgs()

let filename =
    System.IO.Path.Join(__SOURCE_DIRECTORY__, args.[2])

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
