import os
from operator import itemgetter
import sys
import shutil
path = input('input the file storage path:') #输入存放软件仓库的路径



filelist=[]
for root, dir, files in os.walk(path): #将所有文件加入列表中

    for file in files:
       filelist.append(file)



def GetFileDictpath(dir, fileDict): #返回文件绝对路径的字典
    newDir = dir
    if os.path.isfile(dir):  # 如果是文件则添加进 fileDict
        fileDict[os.path.basename(dir)]=dir
    elif os.path.isdir(dir):
        for s in os.listdir(dir):  # 如果是文件夹
            newDir = os.path.join(dir, s)
            GetFileDictpath(newDir, fileDict)
    return fileDict


filedict={}
GetFileDictpath(path,filedict)
file_abspath={}
for key in filedict:
    str_path=filedict[key].split(sep='/')[-2]
    file_abspath[key]=str_path   #生成文件绝对路径


def File_process(fileList,fileDict,path):
    dict1 = {}
    newpath = '/home/s/archive/main' #存放旧包的新路径
    os.makedirs(newpath)
    for i in os.listdir(path):
        os.makedirs(newpath+'/'+i)
    file_name = open(newpath+'_name', 'a')  #将应该退休的旧包写入文档中
    for file in fileList:
            #full_path = os.path.join(root, file)
            str_list = file.split(sep='_') #将deb包的名称通过'-'来分割
            if len(str_list)==3:
                name=str_list[0]+'.'+str_list[2]
                dict1[file]=name
            else :
                shutil.move(fileDict[file],newpath+'/'+file_abspath[file]+'/'+file)
                file_name.writelines(file+'\n')

    list1=sorted(dict1.items(),key=itemgetter(1),reverse=False) #列表文件按字典序顺序排序
    for i in range(len(list1)):
        print(list1[i][0])
    newest=list1[0][1]
    str_list1=list1[0][0].split('_')
    newdeb=str_list1[1]
    newdeb_name=list1[0][0]
    for i in range(1,len(list1)):
        str_list2=list1[i][0].split('_')
        nowdeb=str_list2[1]

        if(newest==list1[i][1]):  #将相同包名的deb包通过上面字符串分割出来的版本号输入到终端进行比较
            r=os.popen('dpkg --compare-versions ' + newdeb + ' gt ' + nowdeb + '\n' + 'echo $?').read().split('\n')[0]
            if(r=='0'):
                file_name.writelines(list1[i][0])
                shutil.move(fileDict[list1[i][0]],newpath+'/'+file_abspath[list1[i][0]]+'/'+list1[i][0])
                file_name.writelines('\n')

            else:
                file_name.writelines(newdeb_name)
                shutil.move(fileDict[newdeb_name],newpath+'/'+file_abspath[newdeb_name]+'/'+newdeb_name)
                file_name.writelines('\n')
                newdeb=nowdeb
                newdeb_name=list1[i][0]
                newest = list1[i][1]

        else:
            newdeb=nowdeb
            newdeb_name=list1[i][0]
            newest=list1[i][1]

File_process(filelist,filedict,path)
