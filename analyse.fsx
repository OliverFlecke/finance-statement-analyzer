#r "nuget: FSharp.Data"

open System
open FSharp.Data

printfn ("Analyzing")

[<Literal>]
let Sample = __SOURCE_DIRECTORY__ + "/sample.csv"

type Transaction = CsvProvider<Sample>

let args = Environment.GetCommandLineArgs()

let filename =
    System.IO.Path.Join(__SOURCE_DIRECTORY__, args.[2])

let data = Transaction.Load(filename)

type Node =
    { category: string
      children: Node list
      items: Transaction.Row list }

let rec insert (root: Node) item (cs: string list) =
    match cs with
    | [] -> { root with items = root.items @ [ item ] }
    | c :: cs' ->
        if List.exists (fun x -> x.category = c) root.children then
            { root with
                children =
                    root.children
                    |> List.map (fun n ->
                        if n.category = c then
                            insert n item cs'
                        else
                            n) }
        else
            let newNode =
                { category = c
                  children = []
                  items = [] }

            { root with children = root.children @ [ insert newNode item cs' ] }

let insertRow (root: Node) (item: Transaction.Row) =
    // printfn $"{root}\nCategories: {item.Category}\n"
    insert root item
    <| (List.ofArray <| item.Category.Split '/')

let performd f node =
    let rec helper d node =
        f d node
        node.children |> Seq.iter (helper (d + 1))

    helper 0 node

let rec perform f node = performd (fun _ x -> f x) node

let root =
    { category = ""
      children = []
      items = [] }

let tree = data.Rows |> Seq.fold insertRow root

performd (fun d n -> printfn $"{new string ('\t', d)}{n.category}") tree


// for r in data.Rows do
//     let date = r.Category
//     printfn $"{date}"
// let date = r..GetColumn "Transaction Date"
// printfn $"{date}"
